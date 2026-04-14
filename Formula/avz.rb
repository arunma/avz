class Avz < Formula
  desc "Blistering-fast Avro CLI tool — a modern replacement for avro-tools and fastavro"
  homepage "https://github.com/arunma/avz"
  url "https://github.com/arunma/avz/archive/refs/tags/v0.1.0.tar.gz"
  # sha256 will be filled after creating the GitHub release
  sha256 "d424740ecd1fd6474207d842c58409ce203072a700e5035a54d1800161dfb778"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on "rust" => :build
  depends_on "cmake" => :build  # needed by aws-lc-sys

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Generate a schema file
    (testpath / "schema.json").write <<~JSON
      {
        "type": "record",
        "name": "Test",
        "fields": [
          {"name": "name", "type": "string"},
          {"name": "value", "type": "int"}
        ]
      }
    JSON

    # Generate random records and verify output
    output = shell_output("#{bin}/avz random --schema #{testpath}/schema.json -n 1 --seed 42")
    assert_match "name", output

    # Convert JSON to Avro and back
    (testpath / "input.json").write '{"name":"test","value":42}'
    system bin / "avz", "fromjson", "--schema", "#{testpath}/schema.json",
           "--output", "#{testpath}/test.avro", "#{testpath}/input.json"
    assert_predicate testpath / "test.avro", :exist?

    output = shell_output("#{bin}/avz cat #{testpath}/test.avro")
    assert_match '"name":"test"', output

    output = shell_output("#{bin}/avz count #{testpath}/test.avro")
    assert_match "1", output.strip
  end
end
