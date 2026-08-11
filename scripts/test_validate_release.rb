# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class ValidateReleaseTest < Minitest::Test
  VALIDATOR = File.expand_path("validate_release.rb", __dir__)

  def test_accepts_matching_tag_manifests_and_changelog
    with_fixture do |root|
      notes = File.join(root, "notes.md")
      result = run_validator(root, "v0.1.0", notes)

      assert result[:status].success?, result[:output]
      release_notes = File.read(notes, encoding: "UTF-8")
      assert_includes release_notes, "## [0.1.0]"
      refute_includes release_notes, "[Unreleased]:"
      refute_includes release_notes, "[0.1.0]:"
    end
  end

  def test_rejects_a_version_mismatch
    with_fixture(version: "0.2.0") do |root|
      result = run_validator(root, "v0.1.0")

      refute result[:status].success?
      assert_includes result[:output], "version does not match"
    end
  end

  private

  def with_fixture(version: "0.1.0")
    Dir.mktmpdir("validate-release-") do |root|
      %w[uuid_factory_cli uuid_factory_core uuid_factory_web].each do |crate|
        path = File.join(root, "crates", crate)
        FileUtils.mkdir_p(path)
        File.write(File.join(path, "Cargo.toml"), "version = \"#{version}\"\n", encoding: "UTF-8")
      end
      File.write(
        File.join(root, "CHANGELOG.md"),
        "# Changelog\n\n## [0.1.0] - 2026-08-12\n\n### Added\n\n- First release.\n\n[Unreleased]: https://example.test/unreleased\n[0.1.0]: https://example.test/v0.1.0\n",
        encoding: "UTF-8"
      )
      yield root
    end
  end

  def run_validator(root, tag, notes = nil)
    command = [RbConfig.ruby, VALIDATOR, "--root", root, "--tag", tag]
    command += ["--notes", notes] if notes
    stdout, stderr, status = Open3.capture3(*command)
    { output: stdout + stderr, status: status }
  end
end
