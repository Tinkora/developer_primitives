#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "optparse"
require "tmpdir"

REQUIRED_PACKAGE_FILES = %w[
  package.json
  uuid_factory_web.js
  uuid_factory_web_bg.wasm
].freeze
OPTIONAL_PACKAGE_FILES = %w[
  LICENSE
  README.md
  uuid_factory_web.d.ts
  uuid_factory_web_bg.wasm.d.ts
].freeze
IGNORED_PACKAGE_FILES = %w[.gitignore].freeze
STATIC_FILES = %w[index.html app.js styles.css].freeze

options = { root: File.expand_path("..", __dir__), wasm_package: nil }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
  parser.on("--wasm-package PATH") { |path| options[:wasm_package] = path }
end.parse!

begin
  root = File.realpath(options[:root])
  source_argument = options[:wasm_package]
  raise "--wasm-package is required" if source_argument.nil? || source_argument.empty?

  source_metadata = File.lstat(source_argument)
  raise "WASM package must be a real directory" unless source_metadata.directory? && !source_metadata.symlink?

  source = File.realpath(source_argument)
  entries = Dir.children(source).sort
  entries.each do |name|
    path = File.join(source, name)
    metadata = File.lstat(path)
    raise "WASM package contains a symbolic link: #{name}" if metadata.symlink?
    raise "WASM package contains a non-file entry: #{name}" unless metadata.file?
    next if (REQUIRED_PACKAGE_FILES + OPTIONAL_PACKAGE_FILES + IGNORED_PACKAGE_FILES).include?(name)

    raise "WASM package contains an unexpected file: #{name}"
  end
  REQUIRED_PACKAGE_FILES.each do |name|
    raise "WASM package is missing #{name}" unless entries.include?(name)
  end

  static = File.join(root, "crates/uuid_factory_web/static")
  STATIC_FILES.each do |name|
    path = File.join(static, name)
    metadata = File.lstat(path)
    raise "Static workbench is missing #{name}" unless metadata.file? && !metadata.symlink?
  end

  output = File.join(root, "dist")
  if File.exist?(output) || File.symlink?(output)
    metadata = File.lstat(output)
    raise "dist must be a real directory" unless metadata.directory? && !metadata.symlink?
  end

  staging = Dir.mktmpdir(".pages-build-", root)
  backup = nil
  begin
    STATIC_FILES.each { |name| FileUtils.copy_file(File.join(static, name), File.join(staging, name)) }
    package_output = File.join(staging, "pkg")
    Dir.mkdir(package_output)
    (REQUIRED_PACKAGE_FILES + OPTIONAL_PACKAGE_FILES).each do |name|
      path = File.join(source, name)
      FileUtils.copy_file(path, File.join(package_output, name)) if File.file?(path)
    end

    if File.exist?(output)
      backup = Dir.mktmpdir(".pages-previous-", root)
      Dir.rmdir(backup)
      File.rename(output, backup)
    end
    File.rename(staging, output)
    staging = nil
    FileUtils.remove_entry_secure(backup) if backup && File.exist?(backup)
    puts "Pages artifact assembled in #{output}."
  ensure
    FileUtils.remove_entry_secure(staging) if staging && File.exist?(staging)
  end
rescue OptionParser::ParseError, SystemCallError, RuntimeError => error
  warn error.message
  exit 1
end
