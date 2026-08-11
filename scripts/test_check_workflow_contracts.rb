# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"
require "yaml"

class CheckWorkflowContractsTest < Minitest::Test
  CHECKER = File.expand_path("check_workflow_contracts.rb", __dir__)
  COMMIT = "e967aed0860957b24daf57e66766713c60b5bcae"

  def test_valid_reusable_references_pass
    with_fixture do |root|
      result = run_checker(root)

      assert result[:status].success?, result[:output]
      assert_includes result[:output], "Workflow contracts passed"
    end
  end

  def test_floating_reusable_reference_fails
    with_fixture(reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must use Tinkora/.github"
    end
  end

  def test_pages_artifacts_require_run_attempt
    with_fixture(include_run_attempt: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must include github.run_attempt"
    end
  end

  def test_release_requires_a_narrow_publication_job
    with_fixture(release_permissions: { "contents" => "read" }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "release job must have contents: write"
    end
  end

  def test_release_requires_all_attestation_permissions
    with_fixture(release_permissions: { "contents" => "write" }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "release job must allow artifact attestations"
    end
  end

  def test_release_requires_provenance_and_sbom_attestations
    with_fixture(attestation_steps: [
      {
        "uses" => "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
        "with" => { "subject-checksums" => "release/SHA256SUMS", "sbom-path" => "release/SBOM.spdx.json" }
      }
    ]) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "release workflow must include provenance and SBOM attestations"
    end
  end

  def test_release_cli_requires_the_job_token
    with_fixture(release_cli_token: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "release CLI must receive github.token"
    end
  end

  def test_all_workflows_require_pinned_external_actions
    with_fixture(extra_action_reference: "actions/checkout@v4") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "external action must use a full commit SHA"
    end
  end

  private

  def with_fixture(
    reference: COMMIT,
    include_run_attempt: true,
    release_permissions: {
      "contents" => "write",
      "attestations" => "write",
      "artifact-metadata" => "write",
      "id-token" => "write"
    },
    attestation_steps: [
      {
        "uses" => "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
        "with" => { "subject-checksums" => "release/provenance-SHA256SUMS" }
      },
      {
        "uses" => "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
        "with" => { "subject-checksums" => "release/sbom-SHA256SUMS", "sbom-path" => "release/SBOM.spdx.json" }
      }
    ],
    release_cli_token: true,
    extra_action_reference: nil
  )
    Dir.mktmpdir("developer-primitives-workflows-") do |root|
      suffix = include_run_attempt ? "-${{ github.run_attempt }}" : ""
      reusable = "Tinkora/.github/.github/workflows"
      write_yaml(root, ".github/workflows/quality.yml", {
        "permissions" => { "contents" => "read" },
        "jobs" => {
          "rust" => { "uses" => "#{reusable}/reusable-rust-quality.yml@#{reference}" },
          "wasm" => { "uses" => "#{reusable}/reusable-wasm-quality.yml@#{reference}" }
        }
      })
      write_yaml(root, ".github/workflows/supply-chain.yml", {
        "permissions" => { "contents" => "read" },
        "jobs" => {
          "audit" => { "uses" => "#{reusable}/reusable-supply-chain.yml@#{reference}" }
        }
      })
      write_yaml(root, ".github/workflows/pages.yml", {
        "permissions" => { "contents" => "read" },
        "jobs" => {
          "assemble" => {
            "if" => "github.ref == 'refs/heads/main'",
            "steps" => [
              { "with" => { "name" => "wasm-package-${{ github.run_id }}#{suffix}" } },
              { "with" => { "name" => "pages-source-${{ github.run_id }}#{suffix}" } }
            ]
          },
          "deploy" => {
            "if" => "github.ref == 'refs/heads/main'",
            "uses" => "#{reusable}/reusable-pages.yml@#{reference}",
            "with" => { "source-artifact-name" => "pages-source-${{ github.run_id }}#{suffix}" }
          }
        }
      })
      release_steps = attestation_steps + [{
        "run" => "gh release create v0.1.0",
        "env" => release_cli_token ? { "GH_TOKEN" => "${{ github.token }}" } : {}
      }]
      write_yaml(root, ".github/workflows/release.yml", {
        "permissions" => { "contents" => "read" },
        "jobs" => {
          "evidence" => { "uses" => "#{reusable}/reusable-release.yml@#{reference}" },
          "release" => {
            "environment" => "release",
            "permissions" => release_permissions,
            "steps" => release_steps
          }
        }
      })
      if extra_action_reference
        write_yaml(root, ".github/workflows/ci.yml", {
          "permissions" => { "contents" => "read" },
          "jobs" => {
            "contracts" => { "steps" => [{ "uses" => extra_action_reference }] }
          }
        })
      end
      File.write(File.join(root, "deny.toml"), "[licenses]\nallow = [\"MIT\"]\n", encoding: "UTF-8")
      yield root
    end
  end

  def write_yaml(root, relative_path, value)
    path = File.join(root, relative_path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, YAML.dump(value), encoding: "UTF-8")
  end

  def run_checker(root)
    stdout, stderr, status = Open3.capture3(RbConfig.ruby, CHECKER, "--root", root)
    { output: stdout + stderr, status: status }
  end
end
