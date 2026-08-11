#!/usr/bin/env ruby

require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
ERRORS = []

def read_utf8(path)
  bytes = path.binread
  if bytes.start_with?("\xEF\xBB\xBF".b)
    ERRORS << "#{path.relative_path_from(ROOT)}: UTF-8 BOM is not allowed"
  end

  text = bytes.force_encoding(Encoding::UTF_8)
  unless text.valid_encoding?
    ERRORS << "#{path.relative_path_from(ROOT)}: invalid UTF-8"
    return ""
  end
  text
end

required = %w[
  README.md
  README.zh-CN.md
  CHANGELOG.md
  CONTRIBUTING.md
  SECURITY.md
  SUPPORT.md
  docs/product_spec.md
  docs/product_spec.zh-CN.md
  skills/mcp-tools.json
]

required.each do |relative|
  path = ROOT.join(relative)
  ERRORS << "#{relative}: required file is missing" unless path.file?
end

markdown_files = ROOT.glob("**/*.md").reject do |path|
  path.each_filename.any? { |part| [".git", "node_modules", "target"].include?(part) }
end

markdown_files.each do |path|
  text = read_utf8(path)
  relative = path.relative_path_from(ROOT)

  text.scan(/(?<!!)\[[^\]]+\]\(([^)]+)\)/).flatten.each do |target|
    destination = target.split(/\s+/, 2).first.delete_prefix("<").delete_suffix(">")
    next if destination.empty? || destination.start_with?("#", "http://", "https://", "mailto:")

    file_target = destination.split("#", 2).first
    resolved = path.dirname.join(file_target).cleanpath
    ERRORS << "#{relative}: broken local link #{destination}" unless resolved.exist?
  end

end

contracts = {
  "README.md" => "[中文](README.zh-CN.md)",
  "README.zh-CN.md" => "[English](README.md)",
  "docs/product_spec.md" => "[中文](product_spec.zh-CN.md)",
  "docs/product_spec.zh-CN.md" => "[English](product_spec.md)"
}

contracts.each do |relative, marker|
  path = ROOT.join(relative)
  next unless path.file?

  ERRORS << "#{relative}: missing bilingual entry #{marker}" unless read_utf8(path).include?(marker)
end

content_contracts = {
  "README.md" => ["tinkora-time", "IANA tzdb 2026c", "UNAMBIGUOUS", "GAP", "FOLD"],
  "README.zh-CN.md" => ["tinkora-time", "IANA tzdb 2026c", "UNAMBIGUOUS", "GAP", "FOLD"],
  "docs/product_spec.md" => ["tinkora-time", "IANA tzdb 2026c", "UNAMBIGUOUS", "GAP", "FOLD"],
  "docs/product_spec.zh-CN.md" => ["tinkora-time", "IANA tzdb 2026c", "UNAMBIGUOUS", "GAP", "FOLD"],
  "THIRD_PARTY_NOTICES.md" => ["Jiff", "jiff-tzdb"]
}

content_contracts.each do |relative, markers|
  path = ROOT.join(relative)
  next unless path.file?

  text = read_utf8(path)
  markers.each do |marker|
    ERRORS << "#{relative}: missing public contract #{marker}" unless text.include?(marker)
  end
end

schema_path = ROOT.join("skills/mcp-tools.json")
if schema_path.file?
  begin
    schema = JSON.parse(read_utf8(schema_path))
    ERRORS << "skills/mcp-tools.json: status must be draft" unless schema["status"] == "draft"
    ERRORS << "skills/mcp-tools.json: runnable must be false" unless schema["runnable"] == false
    ERRORS << "skills/mcp-tools.json: transport must be null" unless schema.key?("transport") && schema["transport"].nil?
  rescue JSON::ParserError => error
    ERRORS << "skills/mcp-tools.json: invalid JSON (#{error.message})"
  end
end

if ERRORS.empty?
  puts "Documentation contracts passed."
  exit 0
end

warn ERRORS.sort.join("\n")
exit 1
