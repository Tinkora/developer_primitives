# frozen_string_literal: true

require "optparse"
require "yaml"

REUSABLE_WORKFLOW_COMMIT = "e967aed0860957b24daf57e66766713c60b5bcae"
REUSABLE_BASE = "Tinkora/.github/.github/workflows"
EXPECTED_CALLS = {
  ".github/workflows/quality.yml" => {
    "rust" => "#{REUSABLE_BASE}/reusable-rust-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}",
    "wasm" => "#{REUSABLE_BASE}/reusable-wasm-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/supply-chain.yml" => {
    "audit" => "#{REUSABLE_BASE}/reusable-supply-chain.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/pages.yml" => {
    "deploy" => "#{REUSABLE_BASE}/reusable-pages.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/release.yml" => {
    "evidence" => "#{REUSABLE_BASE}/reusable-release.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  }
}.freeze
MAIN_CONDITION = "github.ref == 'refs/heads/main'"

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
end.parse!

def values(value)
  case value
  when Hash
    value.values.flat_map { |child| values(child) }
  when Array
    value.flat_map { |child| values(child) }
  when String
    [value]
  else
    []
  end
end

def external_uses(value)
  case value
  when Hash
    own = value["uses"]
    children = value.values.flat_map { |child| external_uses(child) }
    own.is_a?(String) ? [own] + children : children
  when Array
    value.flat_map { |child| external_uses(child) }
  else
    []
  end
end

root = File.expand_path(options[:root])
errors = []
workflows = {}
workflow_root = File.join(root, ".github/workflows")

Dir.glob(File.join(workflow_root, "*.{yml,yaml}")).sort.each do |path|
  relative_path = path.delete_prefix("#{root}/")
  begin
    workflow = YAML.safe_load_file(path, aliases: false)
    raise "workflow must be a mapping" unless workflow.is_a?(Hash)
    workflow.fetch("jobs")
    workflows[relative_path] = workflow
    errors << "#{relative_path}: top-level permissions must be contents: read" unless workflow["permissions"] == { "contents" => "read" }

    external_uses(workflow).uniq.each do |reference|
      next if reference.start_with?("./")
      next if reference.match?(/@[0-9a-f]{40}\z/)

      errors << "#{relative_path}: external action must use a full commit SHA: #{reference}"
    end
  rescue KeyError, NoMethodError, Psych::Exception, RuntimeError => error
    errors << "Invalid workflow #{relative_path}: #{error.message}"
  end
end

EXPECTED_CALLS.each do |relative_path, expected_jobs|
  workflow = workflows[relative_path]
  unless workflow
    errors << "Missing workflow: #{relative_path}"
    next
  end

  jobs = workflow.fetch("jobs")
  expected_jobs.each do |job_name, expected_reference|
    actual = jobs.dig(job_name, "uses")
    errors << "#{relative_path} job #{job_name} must use #{expected_reference}" unless actual == expected_reference
  end
end

pages = workflows[".github/workflows/pages.yml"]
if pages
  jobs = pages.fetch("jobs", {})
  unless %w[assemble deploy].all? { |name| jobs.dig(name, "if") == MAIN_CONDITION }
    errors << ".github/workflows/pages.yml must restrict assembly and deployment to main"
  end
  page_values = values(jobs)
  wasm_artifact = "wasm-package-${{ github.run_id }}-${{ github.run_attempt }}"
  pages_artifact = "pages-source-${{ github.run_id }}-${{ github.run_attempt }}"
  unless page_values.include?(wasm_artifact) && page_values.count(pages_artifact) >= 2
    errors << ".github/workflows/pages.yml artifact names must include github.run_attempt"
  end
end

release = workflows[".github/workflows/release.yml"]
if release
  release_job = release.fetch("jobs", {}).fetch("release", {})
  errors << "release job must have contents: write" unless release_job.dig("permissions", "contents") == "write"
  attestation_permissions = %w[attestations artifact-metadata id-token]
  unless attestation_permissions.all? { |permission| release_job.dig("permissions", permission) == "write" }
    errors << "release job must allow artifact attestations"
  end
  environment = release_job["environment"]
  environment_name = environment.is_a?(Hash) ? environment["name"] : environment
  errors << "release job must use the release environment" unless environment_name == "release"
  attestation_steps = release_job.fetch("steps", []).select do |step|
    step["uses"]&.start_with?("actions/attest@")
  end
  provenance = attestation_steps.any? { |step| !step.fetch("with", {}).key?("sbom-path") }
  sbom = attestation_steps.any? { |step| step.fetch("with", {}).key?("sbom-path") }
  errors << "release workflow must include provenance and SBOM attestations" unless provenance && sbom
end

deny_path = File.join(root, "deny.toml")
unless File.file?(deny_path) && File.read(deny_path, encoding: "UTF-8").include?("[licenses]")
  errors << "deny.toml must define a licenses policy"
end

if errors.empty?
  puts "Workflow contracts passed (commit #{REUSABLE_WORKFLOW_COMMIT})."
  exit 0
end

warn errors.sort.join("\n")
exit 1
