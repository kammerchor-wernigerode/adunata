// Compares versions by the precedence of Semantic Versioning 2.0.0 for the release workflow.
// node semver.mjs greater <a> <b>       exits 0 if a has a higher precedence than b, 1 otherwise
// node semver.mjs highest-stable <v>... prints the highest version without a pre-release part and skips other tags

import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";

const pattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?$/;

export function parse(version) {
  const match = pattern.exec(version);
  if (!match) {
    throw new Error(`${version} is no version of Semantic Versioning without build metadata`);
  }
  return {
    core: match.slice(1, 4).map(BigInt),
    preRelease: match[4] === undefined ? [] : match[4].split("."),
  };
}

function compareIdentifiers(a, b) {
  const aNumeric = /^\d+$/.test(a);
  const bNumeric = /^\d+$/.test(b);
  if (aNumeric && bNumeric) return Math.sign(Number(BigInt(a) - BigInt(b)));
  if (aNumeric) return -1;
  if (bNumeric) return 1;
  return a < b ? -1 : a > b ? 1 : 0;
}

export function compare(a, b) {
  const left = parse(a);
  const right = parse(b);
  for (let i = 0; i < 3; i++) {
    if (left.core[i] !== right.core[i]) return left.core[i] > right.core[i] ? 1 : -1;
  }
  const leftIsStable = left.preRelease.length === 0;
  const rightIsStable = right.preRelease.length === 0;
  if (leftIsStable || rightIsStable) return Number(leftIsStable) - Number(rightIsStable);
  for (let i = 0; i < Math.min(left.preRelease.length, right.preRelease.length); i++) {
    const order = compareIdentifiers(left.preRelease[i], right.preRelease[i]);
    if (order !== 0) return order;
  }
  return Math.sign(left.preRelease.length - right.preRelease.length);
}

export function highestStable(versions) {
  return versions
    .filter((version) => pattern.test(version) && parse(version).preRelease.length === 0)
    .reduce((highest, version) => (highest === undefined || compare(version, highest) > 0 ? version : highest), undefined);
}

function main([command, ...versions]) {
  if (command === "greater" && versions.length === 2) {
    return compare(versions[0], versions[1]) > 0 ? 0 : 1;
  }
  if (command === "highest-stable") {
    const highest = highestStable(versions);
    if (highest !== undefined) console.log(highest);
    return 0;
  }
  console.error("Usage: semver.mjs greater <a> <b> | semver.mjs highest-stable <version>...");
  return 2;
}

if (realpathSync(fileURLToPath(import.meta.url)) === realpathSync(process.argv[1])) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 2;
  }
}
