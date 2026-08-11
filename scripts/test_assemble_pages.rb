# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class AssemblePagesTest < Minitest::Test
  ASSEMBLER = File.expand_path("assemble_pages.rb", __dir__)

  def test_assembles_static_workbench_and_wasm_package
    with_fixture do |root, wasm_package|
      result = run_assembler(root, wasm_package)

      assert result[:status].success?, result[:output]
      assert_equal "<!doctype html>\n", File.read(File.join(root, "dist/index.html"), encoding: "UTF-8")
      assert_equal "export default {};\n", File.read(File.join(root, "dist/pkg/uuid_factory_web.js"), encoding: "UTF-8")
      assert File.file?(File.join(root, "dist/pkg/uuid_factory_web_bg.wasm"))
      refute File.exist?(File.join(root, "dist/pkg/.gitignore"))
      refute File.exist?(File.join(root, "dist/sentinel.txt"))
    end
  end

  def test_rejects_unexpected_wasm_output_without_replacing_existing_site
    with_fixture do |root, wasm_package|
      File.write(File.join(wasm_package, "unexpected.txt"), "no\n", encoding: "UTF-8")

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "unexpected file"
      assert_equal "previous\n", File.read(File.join(root, "dist/sentinel.txt"), encoding: "UTF-8")
    end
  end

  private

  def with_fixture
    Dir.mktmpdir("assemble-pages-") do |root|
      static = File.join(root, "crates/uuid_factory_web/static")
      wasm_package = File.join(root, "wasm-package")
      FileUtils.mkdir_p(static)
      FileUtils.mkdir_p(wasm_package)
      FileUtils.mkdir_p(File.join(root, "dist"))
      %w[index.html app.js styles.css].each do |name|
        File.write(File.join(static, name), name == "index.html" ? "<!doctype html>\n" : "\n", encoding: "UTF-8")
      end
      File.write(File.join(root, "dist/sentinel.txt"), "previous\n", encoding: "UTF-8")
      File.write(File.join(wasm_package, "package.json"), "{}\n", encoding: "UTF-8")
      File.write(File.join(wasm_package, ".gitignore"), "*.d.ts\n", encoding: "UTF-8")
      File.write(File.join(wasm_package, "uuid_factory_web.js"), "export default {};\n", encoding: "UTF-8")
      File.binwrite(File.join(wasm_package, "uuid_factory_web_bg.wasm"), "\0asm")
      yield root, wasm_package
    end
  end

  def run_assembler(root, wasm_package)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      ASSEMBLER,
      "--root",
      root,
      "--wasm-package",
      wasm_package
    )
    { output: stdout + stderr, status: status }
  end
end
