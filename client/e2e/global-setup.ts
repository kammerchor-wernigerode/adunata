import { type ChildProcess, execFileSync, spawn } from 'node:child_process'
import { createHash } from 'node:crypto'
import { mkdirSync, openSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:net'
import path from 'node:path'

const client = path.resolve(import.meta.dirname, '..')
const root = git('rev-parse', '--show-toplevel')
const project = `adunata-e2e-${createHash('sha256').update(root).digest('hex').slice(0, 12)}`
const results = path.join(client, 'test-results')

function git(...args: string[]): string {
  return execFileSync('git', args, { cwd: client, encoding: 'utf8' }).trim()
}

function compose(...args: string[]): string {
  return execFileSync('docker', ['compose', '-p', project, ...args], {
    cwd: root,
    encoding: 'utf8',
    env: { ...process.env, POSTGRES_PORT: '0', MAILPIT_SMTP_PORT: '0', MAILPIT_UI_PORT: '0' },
    stdio: ['ignore', 'pipe', 'inherit'],
  }).trim()
}

/** The host port that Compose bound to a port of a service, such as 55001 for 127.0.0.1:55001. */
function hostPort(service: string, port: number): number {
  return Number(compose('port', service, String(port)).split(':').pop())
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer()
    server.once('error', reject)
    server.listen(0, '127.0.0.1', () => {
      const address = server.address()
      server.close(() => resolve(typeof address === 'object' && address ? address.port : 0))
    })
  })
}

/** The binary from ADUNATA_E2E_SERVER, otherwise a release build of the client and the server in this checkout. */
function serverBinary(): string {
  const given = process.env.ADUNATA_E2E_SERVER
  if (given) return path.resolve(given)
  execFileSync('npm', ['run', 'build'], { cwd: client, stdio: 'inherit' })
  execFileSync('cargo', ['build', '--release', '--locked'], { cwd: path.join(root, 'server'), stdio: 'inherit' })
  return path.join(root, 'server', 'target', 'release', 'adunata')
}

async function untilAnswers(url: string, server: ChildProcess): Promise<void> {
  for (let attempt = 0; attempt < 300; attempt++) {
    if (server.exitCode !== null) throw new Error(`the server exited with ${server.exitCode}, see ${results}/server.log`)
    const answer = await fetch(url).catch(() => undefined)
    if (answer?.ok) return
    await new Promise((resolve) => setTimeout(resolve, 100))
  }
  throw new Error(`the server did not answer at ${url}, see ${results}/server.log`)
}

/** Starts fresh services from compose.yaml as a Compose project of this checkout and a server with a cleared environment, and removes both afterwards. */
export default async function globalSetup(): Promise<() => Promise<void>> {
  const binary = serverBinary()
  compose('down', '--volumes')
  compose('up', '--detach', '--wait')
  const postgres = hostPort('postgres', 5432)
  const smtp = hostPort('mailpit', 1025)
  const mailpit = hostPort('mailpit', 8025)
  const http = await freePort()
  const url = `http://localhost:${http}`

  mkdirSync(results, { recursive: true })
  const log = openSync(path.join(results, 'server.log'), 'w')
  const server = spawn(binary, ['serve'], {
    cwd: results,
    stdio: ['ignore', log, log],
    env: {
      ADNT_DATABASE_URL: `postgres://adunata:adunata@127.0.0.1:${postgres}/adunata`,
      ADNT_HTTP_LISTEN_ADDRESS: `127.0.0.1:${http}`,
      ADNT_HTTP_PUBLIC_URL: url,
      ADNT_SMTP_HOST: '127.0.0.1',
      ADNT_SMTP_PORT: String(smtp),
      ADNT_SMTP_ENCRYPTION: 'none',
      ADNT_MAIL_SENDER: 'Adunata <adunata@example.org>',
      ADNT_LOCALIZATION_TIME_ZONE: 'Europe/Berlin',
      ADNT_BOOTSTRAP_ORGANIZER_MAIL: 'organizer@example.org',
      ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME: 'Ada',
      ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME: 'Lovelace',
    },
  })

  const teardown = async () => {
    server.kill()
    writeFileSync(path.join(results, 'services.log'), compose('logs', '--no-color'))
    compose('down', '--volumes')
  }
  try {
    await untilAnswers(`${url}/api/health`, server)
  } catch (error) {
    await teardown()
    throw error
  }
  process.env.ADUNATA_E2E_URL = url
  process.env.ADUNATA_E2E_MAILPIT_URL = `http://127.0.0.1:${mailpit}`
  return teardown
}
