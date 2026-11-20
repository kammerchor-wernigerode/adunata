import assert from "node:assert/strict";
import { test } from "node:test";
import { compare, highestStable, parse } from "./semver.mjs";

test("orders versions by the precedence of Semantic Versioning", () => {
  const ordered = [
    "0.0.0",
    "0.1.0",
    "1.0.0-alpha",
    "1.0.0-alpha.1",
    "1.0.0-alpha.beta",
    "1.0.0-beta",
    "1.0.0-beta.2",
    "1.0.0-beta.11",
    "1.0.0-rc.1",
    "1.0.0",
    "1.2.0",
    "1.10.0",
    "2.0.0",
  ];
  for (let i = 0; i < ordered.length - 1; i++) {
    assert.equal(compare(ordered[i + 1], ordered[i]), 1, `${ordered[i + 1]} > ${ordered[i]}`);
    assert.equal(compare(ordered[i], ordered[i + 1]), -1, `${ordered[i]} < ${ordered[i + 1]}`);
  }
  assert.equal(compare("1.2.0-rc.1", "1.2.0-rc.1"), 0);
});

test("rejects build metadata and malformed versions", () => {
  for (const version of ["1.2.0+1", "v1.2.0", "1.2", "01.2.0", "1.2.0-", "1.2.0-01"]) {
    assert.throws(() => parse(version), version);
  }
});

test("picks the highest version without a pre-release part", () => {
  assert.equal(highestStable(["1.2.0", "1.10.0", "1.3.0", "2.0.0-rc.1", "test", "1.0"]), "1.10.0");
  assert.equal(highestStable(["1.0.0-rc.1"]), undefined);
});
