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

ci = workflows[".github/workflows/ci.yml"]
ci_values = ci ? values(ci.fetch("jobs", {})) : []
unless ci_values.any? { |value| value.include?("ruby scripts/check_workflow_contracts.rb") }
  errors << "CI must run scripts/check_workflow_contracts.rb"
end

release = workflows[".github/workflows/release.yml"]
if release
  release_jobs = release.fetch("jobs", {})
  cli_job = release_jobs.fetch("cli", {})
  cli_steps = cli_job.fetch("steps", [])
  cli_commands = cli_steps.filter_map { |step| step["run"] if step["run"].is_a?(String) }
  build_command = cli_commands.find { |command| command.match?(/\bcargo\b.*\bbuild\b/) }
  cli_crates = %w[uuid_factory_cli timestamp_zone_cli]
  unless build_command && cli_crates.all? { |crate| build_command.match?(/(?:^|\s)-p\s+#{Regexp.escape(crate)}(?:\s|$)/) }
    errors << "release CLI build must include uuid_factory_cli and timestamp_zone_cli"
  end
  cli_binaries = %w[tinkora-id tinkora-time]
  unless cli_binaries.all? { |binary| cli_commands.any? { |command| command.include?(binary) && command.include?("--version") } }
    errors << "release CLI verification must include tinkora-id and tinkora-time"
  end
  archive_steps = cli_steps.select do |step|
    name = step.fetch("name", "")
    command = step["run"]
    command.is_a?(String) && (name.start_with?("Archive ") || command.match?(/archive_?[Ss]tem\s*=/))
  end
  archive_stems = [
    "tinkora-developer-primitives-${VERSION}-${TARGET}",
    "tinkora-developer-primitives-$VERSION-$TARGET",
    "tinkora-developer-primitives-$env:VERSION-$env:TARGET"
  ]
  unless archive_steps.any? && archive_steps.all? { |step| cli_binaries.all? { |binary| step["run"].include?(binary) } }
    errors << "release CLI archives must include tinkora-id and tinkora-time"
  end
  archive_step_names = archive_steps.map { |step| step.fetch("name", "") }
  unless %w[Archive\ Unix\ CLI Archive\ Windows\ CLI].all? { |name| archive_step_names.include?(name) }
    errors << "release CLI job must include Unix and Windows archive steps"
  end
  linked_docs = %w[README.md README.zh-CN.md docs/product_spec.md docs/product_spec.zh-CN.md]
  unless archive_steps.any? && archive_steps.all? { |step| linked_docs.all? { |path| step["run"].include?(path) } }
    errors << "release CLI archives must include linked bilingual documentation"
  end
  unless archive_steps.any? && archive_steps.all? { |step| archive_stems.any? { |stem| step["run"].include?(stem) } }
    errors << "release CLI archives must use the tinkora-developer-primitives VERSION and TARGET stem"
  end
  archive_steps.each do |step|
    name = step.fetch("name", "")
    command = step["run"]
    if name.include?("Unix") && !command.include?(".tar.gz")
      errors << "release Unix CLI archive must use tar.gz"
    elsif name.include?("Windows") && !command.include?(".zip")
      errors << "release Windows CLI archive must use zip"
    end
  end
  matrix = cli_job.dig("strategy", "matrix", "include")
  if matrix.is_a?(Array)
    expected_platforms = %w[linux macos windows]
    actual_platforms = matrix.filter_map { |entry| entry["platform"] }
    unless actual_platforms.sort == expected_platforms.sort
      errors << "release CLI matrix must include linux, macos, and windows"
    end
    extensions_valid = matrix.all? do |entry|
      entry["extension"] == (entry["platform"] == "windows" ? "zip" : "tar.gz")
    end
    errors << "release CLI matrix must use zip for Windows and tar.gz elsewhere" unless extensions_valid
  else
    errors << "release CLI matrix must include linux, macos, and windows"
  end
  upload_step = cli_steps.find { |step| step.fetch("name", "") == "Upload CLI archive" }
  if upload_step
    expected_path = "${{ runner.temp }}/tinkora-developer-primitives-${{ needs.metadata.outputs.version }}-${{ matrix.target }}.${{ matrix.extension }}"
    errors << "release CLI upload path must use the shared archive stem" unless upload_step.dig("with", "path") == expected_path
  else
    errors << "release CLI job must upload the platform archive"
  end

  release_job = release_jobs.fetch("release", {})
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
  publication_step = release_job.fetch("steps", []).find do |step|
    step["run"]&.include?("gh release create")
  end
  unless publication_step&.dig("env", "GH_TOKEN") == "${{ github.token }}"
    errors << "release CLI must receive github.token"
  end
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
