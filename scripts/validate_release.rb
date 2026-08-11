#!/usr/bin/env ruby
# frozen_string_literal: true

require "optparse"

options = { root: File.expand_path("..", __dir__), tag: nil, notes: nil }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
  parser.on("--tag TAG") { |tag| options[:tag] = tag }
  parser.on("--notes PATH") { |path| options[:notes] = path }
end.parse!

begin
  root = File.realpath(options[:root])
  tag = options[:tag]
  raise "--tag is required" unless tag
  match = /\Av(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)\z/.match(tag)
  raise "tag must use vMAJOR.MINOR.PATCH" unless match

  version = tag.delete_prefix("v")
  %w[uuid_factory_cli uuid_factory_core uuid_factory_web].each do |crate|
    manifest = File.read(File.join(root, "crates", crate, "Cargo.toml"), encoding: "UTF-8")
    raise "#{crate} version does not match #{tag}" unless manifest.match?(/^version = "#{Regexp.escape(version)}"$/)
  end

  changelog = File.read(File.join(root, "CHANGELOG.md"), encoding: "UTF-8")
  heading = "## [#{version}]"
  start = changelog.index(heading)
  raise "CHANGELOG.md is missing #{heading}" unless start
  following_heading = changelog.index(/^## \[/, start + heading.length)
  link_definitions = changelog.index(/^\[[^\]]+\]:\s+\S+/, start + heading.length)
  section_end = [following_heading, link_definitions, changelog.length].compact.min
  release_notes = changelog[start...section_end].strip
  raise "release notes are empty" if release_notes.empty?

  if options[:notes]
    File.write(options[:notes], "# #{tag}\n\n#{release_notes}\n", encoding: "UTF-8")
  end
  puts "Release metadata validated for #{tag}."
rescue OptionParser::ParseError, SystemCallError, RuntimeError => error
  warn error.message
  exit 1
end
