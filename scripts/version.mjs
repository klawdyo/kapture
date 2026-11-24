// Versionamento semântico automático a partir de Conventional Commits.
// Regras em docs/versioning.md. Só usa `git` (child_process), sem dependências.
// Funções puras no topo; E/S (git, arquivos, CLI) na seção final.

// Configuração simples: como tratar `!` / BREAKING CHANGE.
// 'minor' (padrão, sobe minor e zera patch), 'major' ou 'none' (ignora o marcador).
export const BREAKING = 'minor';

export const TYPES = [
  'feat', 'fix', 'perf', 'refactor', 'docs', 'style', 'test', 'build', 'ci', 'chore', 'revert',
];

// Títulos das seções do changelog, na ordem de exibição.
export const SECTIONS = [
  ['breaking', 'Mudanças incompatíveis'],
  ['feat', 'Novidades'],
  ['fix', 'Correções'],
  ['perf', 'Desempenho'],
  ['refactor', 'Refatoração'],
  ['docs', 'Documentação'],
  ['test', 'Testes'],
  ['build', 'Build'],
  ['ci', 'CI'],
  ['style', 'Estilo'],
  ['chore', 'Manutenção'],
  ['revert', 'Reversões'],
];

const SUBJECT = /^([a-z]+)(?:\(([^)\s]+)\))?(!)?: (\S.*)$/;
const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;
const TAG = /^v(\d+\.\d+\.\d+)$/;

// Commit de merge: mais de um pai ou assunto padrão do git/GitHub.
export function isMerge(c) {
  return (c.parents?.length ?? 0) > 1 || /^Merge /.test(c.subject);
}

// Interpreta um commit { hash, subject, body, parents }.
export function parseCommit(c) {
  if (isMerge(c)) return { ...c, merge: true, valid: false };
  const m = SUBJECT.exec(c.subject);
  if (!m || !TYPES.includes(m[1])) return { ...c, merge: false, valid: false };
  const breaking = Boolean(m[3]) || /^BREAKING[ -]CHANGE: /m.test(c.body ?? '');
  return {
    ...c, merge: false, valid: true, type: m[1], scope: m[2] ?? null, breaking, desc: m[4],
  };
}

export function parseVersion(v) {
  const m = SEMVER.exec(v);
  if (!m) throw new Error(`Versão inválida: ${v}`);
  return { major: +m[1], minor: +m[2], patch: +m[3] };
}

export const fmtVersion = (v) => `${v.major}.${v.minor}.${v.patch}`;

// Efeito de um commit já interpretado: 'major' | 'minor' | 'patch' | null.
export function bumpOf(p, breaking = BREAKING) {
  if (!p.valid) return null;
  if (p.breaking && breaking !== 'none') return breaking;
  if (p.type === 'feat') return 'minor';
  if (p.type === 'fix') return 'patch';
  return null;
}

// Soma sequencial: commits do mais antigo ao mais novo.
export function nextVersion(base, commits, breaking = BREAKING) {
  const v = parseVersion(base);
  for (const c of commits) {
    const b = bumpOf(parseCommit(c), breaking);
    if (b === 'major') { v.major += 1; v.minor = 0; v.patch = 0; }
    else if (b === 'minor') { v.minor += 1; v.patch = 0; }
    else if (b === 'patch') v.patch += 1;
  }
  return fmtVersion(v);
}

// Commits fora do padrão (merges não contam).
export function invalidCommits(commits) {
  return commits.map(parseCommit).filter((p) => !p.merge && !p.valid);
}

// Última tag vX.Y.Z (maior versão) de uma lista de nomes de tag.
export function lastVersion(tags) {
  const vs = tags
    .map((t) => TAG.exec(t)?.[1])
    .filter(Boolean)
    .sort((a, b) => {
      const x = parseVersion(a);
      const y = parseVersion(b);
      return y.major - x.major || y.minor - x.minor || y.patch - x.patch;
    });
  return vs[0] ?? null;
}

// Notas em markdown, agrupadas por tipo, na ordem em que os commits entraram.
export function generateNotes(commits, version, date = null) {
  const parsed = commits.map(parseCommit).filter((p) => p.valid);
  const groups = new Map();
  const add = (k, p) => groups.set(k, [...(groups.get(k) ?? []), p]);
  for (const p of parsed) {
    if (p.breaking) add('breaking', p);
    add(p.type, p);
  }
  const head = `## v${version}${date ? ` (${date})` : ''}`;
  const out = [head];
  for (const [key, title] of SECTIONS) {
    const items = groups.get(key);
    if (!items) continue;
    out.push('', `### ${title}`, '');
    for (const p of items) {
      const scope = p.scope ? `**${p.scope}:** ` : '';
      const sha = p.hash ? ` (${p.hash.slice(0, 7)})` : '';
      out.push(`- ${scope}${p.desc}${sha}`);
    }
  }
  if (out.length === 1) out.push('', 'Sem mudanças relevantes.');
  return out.join('\n') + '\n';
}

// Troca a primeira `version = "..."` da seção [package] de um Cargo.toml.
export function setCargoVersion(text, version) {
  return text.replace(/(\[package\][^[]*?\nversion\s*=\s*")[^"]*(")/, `$1${version}$2`);
}

// Troca o campo version de um JSON, mantendo a indentação de 2 espaços.
export function setJsonVersion(text, version) {
  return JSON.stringify({ ...JSON.parse(text), version }, null, 2) + '\n';
}

// Interpreta a saída de `git log` com separadores \x1f (campo) e \x1e (registro).
export function parseLog(raw) {
  return raw
    .split('\x1e')
    .map((r) => r.replace(/^\n/, ''))
    .filter((r) => r.trim())
    .map((r) => {
      const [hash, parents, subject, body] = r.split('\x1f');
      return { hash, parents: parents ? parents.split(' ') : [], subject, body };
    });
}

/* v8 ignore start */
// ---- E/S: git, arquivos e CLI ----
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const git = (...args) => execFileSync('git', args, { encoding: 'utf8', maxBuffer: 64 << 20 });

function readLog(range) {
  const fmt = '%H%x1f%P%x1f%s%x1f%b%x1e';
  const args = ['log', '--reverse', `--format=${fmt}`];
  return parseLog(git(...(range ? [...args, range] : [...args, 'HEAD'])));
}

// Última versão alcançável a partir de HEAD e os commits desde ela.
function context() {
  const tags = git('tag', '--merged', 'HEAD', '--list', 'v*').split('\n').filter(Boolean);
  const base = lastVersion(tags);
  return { base: base ?? '0.0.0', commits: readLog(base ? `v${base}..HEAD` : null) };
}

function applyVersion(version) {
  const edit = (f, fn) => writeFileSync(f, fn(readFileSync(f, 'utf8'), version));
  edit('src-tauri/tauri.conf.json', setJsonVersion);
  edit('package.json', setJsonVersion);
  edit('src-tauri/Cargo.toml', setCargoVersion);
  edit('crates/cap-core/Cargo.toml', setCargoVersion);
}

function main(argv) {
  const [cmd, arg] = argv;
  if (cmd === 'next') {
    const { base, commits } = context();
    console.log(nextVersion(base, commits));
  } else if (cmd === 'base') {
    console.log(context().base);
  } else if (cmd === 'notes') {
    const { base, commits } = context();
    const date = new Date().toISOString().slice(0, 10);
    process.stdout.write(generateNotes(commits, nextVersion(base, commits), date));
  } else if (cmd === 'check') {
    if (!arg) throw new Error('Uso: version.mjs check <range>');
    const bad = invalidCommits(readLog(arg));
    for (const p of bad) console.error(`Fora do padrão: ${p.hash.slice(0, 7)} ${p.subject}`);
    if (bad.length) {
      console.error('Use o formato tipo(escopo)!: descrição. Veja docs/versioning.md.');
      process.exit(1);
    }
    console.log('Todos os commits seguem o padrão.');
  } else if (cmd === 'apply') {
    if (!SEMVER.test(arg ?? '')) throw new Error('Uso: version.mjs apply X.Y.Z');
    applyVersion(arg);
  } else {
    console.error('Uso: version.mjs next | base | notes | check <range> | apply X.Y.Z');
    process.exit(2);
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
/* v8 ignore stop */
