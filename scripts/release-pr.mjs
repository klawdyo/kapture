// Prepara o PR da dev para a main: calcula a versão EXATAMENTE como o release.yml fará na main
// e imprime o título, as notas e a lista de commits. Uso: node scripts/release-pr.mjs [dev-ref]
// Só lê o Git (nada é alterado). Rode `git fetch --tags origin` antes, para ter as tags do remoto.
import { execFileSync } from 'node:child_process'
import { generateNotes, lastVersion, nextVersion, parseLog } from './version.mjs'

// Referência do que vai ser mesclado (padrão: a dev do remoto) e da main do remoto.
const head = process.argv[2] ?? 'origin/dev'
const main = 'origin/main'

const git = (...args) => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 << 20 })

// A base é a maior tag vX.Y.Z JÁ ALCANÇÁVEL a partir da main: o release calcula assim, partindo da
// main depois do merge. Partir do HEAD da dev engana, porque a dev pode não enxergar a última tag
// (o commit de merge que recebe a tag só existe na main).
const tags = git('tag', '--merged', main, '--list', 'v*').split('\n').filter(Boolean)
const base = lastVersion(tags)
if (!base) {
  console.error('Nenhuma tag vX.Y.Z alcançável a partir de origin/main. Rode: git fetch --tags origin')
  process.exit(1)
}

// Commits que a main ainda não tem, do mais antigo ao mais novo (mesma ordem do cálculo).
const fmt = '%H%x1f%P%x1f%s%x1f%b%x1e'
const commits = parseLog(git('log', '--reverse', `--format=${fmt}`, `v${base}..${head}`))
const version = nextVersion(base, commits)
const date = new Date().toISOString().slice(0, 10)

if (version === base) {
  console.error(`Sem commits feat/fix desde v${base}: o release não publicaria nada.`)
  process.exit(2)
}

// Lista de todos os commits (inclusive merges), com hash curto e assunto.
const list = git('log', '--reverse', '--format=- `%h` %s', `v${base}..${head}`).trim()
const total = list.split('\n').length

console.log(`TÍTULO: v${version}`)
console.log(`BASE: v${base} (${head} contra ${main})\n`)
console.log(generateNotes(commits, version, date).trimEnd())
console.log(`\n## Todos os commits desde a v${base} (${total})\n`)
console.log(list)
