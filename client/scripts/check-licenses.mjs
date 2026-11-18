import { readFileSync } from "node:fs";

const lockFile = new URL("../package-lock.json", import.meta.url);
const denyFile = new URL("../../server/deny.toml", import.meta.url);

function allowedLicenses() {
  const list = readFileSync(denyFile, "utf8").match(/^allow = \[([^\]]*)\]/m);
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

function problems() {
  const allowed = allowedLicenses();
  const { packages } = JSON.parse(readFileSync(lockFile, "utf8"));
  return Object.entries(packages)
    .filter(([path]) => path !== "")
    .flatMap(([path, { license }]) => {
      const name = path.replace(/^.*node_modules\//, "");
      if (typeof license !== "string") return [`${name} (${path}) declares no license`];
      try {
        return satisfies(license, allowed) ? [] : [`${name} (${path}) has license ${license}, which deny.toml does not allow`];
      } catch (error) {
        return [`${name} (${path}): ${error.message}`];
      }
    });
}

const found = problems();
if (found.length > 0) {
  console.error(found.join("\n"));
  process.exit(1);
}
console.log("licenses ok");
