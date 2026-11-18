import { readFileSync } from "node:fs";

const denyFile = new URL("../../server/deny.toml", import.meta.url);

function allowedLicenses() {
  const withoutComments = readFileSync(denyFile, "utf8").replace(/#.*$/gm, "");
  const list = withoutComments.match(/^allow = \[([^\]]*)\]/m);
  if (!list) {
    throw new Error(`${denyFile.pathname} has no list "allow"`);
  }
  return new Set([...list[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]));
}

function tokenize(expression) {
  return expression.match(/\(|\)|[^\s()]+/g) ?? [];
}

/** Tells whether an SPDX expression of identifiers, OR, AND and parentheses is satisfied by the allowed licenses. */
function satisfies(expression, allowed) {
  const tokens = tokenize(expression);
  let position = 0;

  function fail(reason) {
    throw new Error(`${reason} in "${expression}"`);
  }

  function next() {
    return tokens[position++];
  }

  function either() {
    let result = both();
    while (tokens[position] === "OR") {
      next();
      result = both() || result;
    }
    return result;
  }

  function both() {
    let result = license();
    while (tokens[position] === "AND") {
      next();
      result = license() && result;
    }
    return result;
  }

  function license() {
    const token = next();
    if (token === "(") {
      const result = either();
      if (next() !== ")") fail("missing closing parenthesis");
      return result;
    }
    if (token === undefined || !/^[A-Za-z0-9.-]+$/.test(token)) fail(`unsupported token "${token ?? "end"}"`);
    if (token === "OR" || token === "AND") fail(`misplaced operator ${token}`);
    return allowed.has(token);
  }

  const result = either();
  if (position < tokens.length) fail(`unsupported token "${tokens[position]}"`);
  return result;
}

/** Checks the output of `pnpm licenses list --json`, which groups the installed packages by their license. */
function problems(report) {
  const allowed = allowedLicenses();
  return Object.entries(report).flatMap(([license, packages]) => {
    const names = packages.map((entry) => `${entry.name} ${entry.versions.join(", ")}`);
    if (license === "Unknown") return names.map((name) => `${name} declares no license`);
    try {
      return satisfies(license, allowed) ? [] : names.map((name) => `${name} has license ${license}, which deny.toml does not allow`);
    } catch (error) {
      return names.map((name) => `${name}: ${error.message}`);
    }
  });
}

const reportFile = process.argv[2];
if (!reportFile) {
  console.error("usage: check-licenses.mjs <output of pnpm licenses list --json>");
  process.exit(2);
}
const found = problems(JSON.parse(readFileSync(reportFile, "utf8")));
if (found.length > 0) {
  console.error(found.join("\n"));
  process.exit(1);
}
console.log("licenses ok");
