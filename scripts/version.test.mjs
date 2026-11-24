// Testes das regras de versionamento (docs/versioning.md).
import { describe, expect, it } from 'vitest';
import {
  bumpOf, fmtVersion, generateNotes, invalidCommits, isMerge, lastVersion, nextVersion,
  parseCommit, parseLog, parseVersion, setCargoVersion, setJsonVersion,
} from './version.mjs';

const c = (subject, extra = {}) => ({ hash: 'abcdef123456', subject, body: '', parents: ['p1'], ...extra });

describe('parseCommit', () => {
  it('lê tipo, escopo e descrição', () => {
    expect(parseCommit(c('feat(ui): botão novo'))).toMatchObject({
      valid: true, type: 'feat', scope: 'ui', breaking: false, desc: 'botão novo',
    });
  });
  it('sem escopo', () => {
    expect(parseCommit(c('fix: corrige')).scope).toBeNull();
  });
  it('detecta ! e BREAKING CHANGE no corpo', () => {
    expect(parseCommit(c('feat!: troca api')).breaking).toBe(true);
    expect(parseCommit(c('fix(x)!: y')).breaking).toBe(true);
    expect(parseCommit(c('fix: y', { body: 'a\n\nBREAKING CHANGE: muda' })).breaking).toBe(true);
  });
  it('commit fora do padrão é inválido', () => {
    for (const s of ['atualiza coisas', 'feature: x', 'feat:sem espaço', 'Feat: x', 'feat: ']) {
      expect(parseCommit(c(s)).valid).toBe(false);
    }
  });
  it('corpo ausente é aceito', () => {
    expect(parseCommit({ hash: 'a', subject: 'fix: a', parents: [] }).valid).toBe(true);
  });
  it('merge é reconhecido por pais ou assunto', () => {
    expect(isMerge(c('x', { parents: ['a', 'b'] }))).toBe(true);
    expect(isMerge(c('Merge pull request #2 from a/b'))).toBe(true);
    expect(isMerge({ subject: 'fix: a' })).toBe(false);
    expect(parseCommit(c('Merge branch dev')).merge).toBe(true);
  });
});

describe('nextVersion', () => {
  it('exemplo da spec: feat zera patch e soma sequencial', () => {
    const log = [c('fix: a'), c('fix: b'), c('feat: c'), c('fix: d'), c('docs: e'), c('feat(x): f'), c('chore: g')];
    // 1.4.2 -> fix 1.4.3 -> fix 1.4.4 -> feat 1.5.0 -> fix 1.5.1 -> feat 1.6.0
    expect(nextVersion('1.4.2', log)).toBe('1.6.0');
  });
  it('a ordem importa: fix depois de feat sobe patch, feat depois de fix zera', () => {
    expect(nextVersion('1.0.0', [c('feat: a'), c('fix: b')])).toBe('1.1.1');
    expect(nextVersion('1.0.0', [c('fix: b'), c('feat: a')])).toBe('1.1.0');
  });
  it('merge é ignorado, commits reais contam', () => {
    const log = [c('feat: a'), c('Merge pull request #1', { parents: ['a', 'b'] }), c('fix: b')];
    expect(nextVersion('0.1.0', log)).toBe('0.2.1');
  });
  it('sem tag: base 0.0.0 e todo o histórico', () => {
    expect(nextVersion('0.0.0', [c('feat: a'), c('feat: b'), c('fix: c')])).toBe('0.2.1');
  });
  it('sem commits novos ou só tipos neutros mantém a versão', () => {
    expect(nextVersion('1.2.3', [])).toBe('1.2.3');
    expect(nextVersion('1.2.3', [c('docs: a'), c('chore: b'), c('ci: c'), c('refactor: d')])).toBe('1.2.3');
  });
  it('commits fora do padrão são ignorados no cálculo', () => {
    expect(nextVersion('1.0.0', [c('mexi em coisa'), c('fix: a')])).toBe('1.0.1');
  });
  it('breaking: minor por padrão, configurável', () => {
    const log = [c('fix: a'), c('refactor!: b')];
    expect(nextVersion('0.3.0', log)).toBe('0.4.0');
    expect(nextVersion('0.3.0', log, 'major')).toBe('1.0.0');
    expect(nextVersion('0.3.0', log, 'none')).toBe('0.3.1');
    expect(bumpOf({ valid: true, type: 'docs', breaking: false })).toBeNull();
    expect(bumpOf({ valid: false })).toBeNull();
  });
});

describe('versões e tags', () => {
  it('parse e formato', () => {
    expect(fmtVersion(parseVersion('1.20.3'))).toBe('1.20.3');
    expect(() => parseVersion('1.2')).toThrow();
  });
  it('lastVersion escolhe a maior vX.Y.Z e ignora o resto', () => {
    expect(lastVersion(['v1.2.0', 'v1.10.0', 'v1.9.9', 'vfoo', 'v2.0.0-rc1'])).toBe('1.10.0');
    expect(lastVersion(['v2.0.0', 'v1.9.0'])).toBe('2.0.0');
    expect(lastVersion(['v1.0.1', 'v1.0.2'])).toBe('1.0.2');
    expect(lastVersion([])).toBeNull();
  });
});

describe('invalidCommits', () => {
  it('lista só os fora do padrão, sem merges', () => {
    const bad = invalidCommits([c('feat: a'), c('ruim'), c('Merge x', { parents: ['a', 'b'] })]);
    expect(bad.map((p) => p.subject)).toEqual(['ruim']);
  });
});

describe('generateNotes', () => {
  it('agrupa por tipo, com escopo, hash, data e breaking', () => {
    const md = generateNotes(
      [c('fix: b'), c('feat(ui): a'), c('feat!: c'), c('ruim'), c('Merge x', { parents: ['a', 'b'] })],
      '1.1.0',
      '2026-01-02',
    );
    expect(md).toContain('## v1.1.0 (2026-01-02)');
    expect(md.indexOf('Mudanças incompatíveis')).toBeLessThan(md.indexOf('Novidades'));
    expect(md.indexOf('Novidades')).toBeLessThan(md.indexOf('Correções'));
    expect(md).toContain('- **ui:** a (abcdef1)');
    expect(md).not.toContain('ruim');
  });
  it('sem hash e sem data, e vazio', () => {
    expect(generateNotes([{ subject: 'fix: a', parents: [] }], '0.0.1')).toContain('- a\n');
    expect(generateNotes([], '0.0.0')).toContain('Sem mudanças relevantes.');
  });
});

describe('arquivos e log', () => {
  it('setCargoVersion troca só a versão do pacote', () => {
    const t = '[package]\nname = "a"\nversion = "0.1.0"\n\n[dependencies]\nx = { version = "2" }\n';
    expect(setCargoVersion(t, '1.2.3')).toBe(t.replace('0.1.0', '1.2.3'));
  });
  it('setJsonVersion mantém os outros campos', () => {
    expect(JSON.parse(setJsonVersion('{"a":1,"version":"0.1.0"}', '2.0.0'))).toEqual({ a: 1, version: '2.0.0' });
  });
  it('parseLog lê registros do git', () => {
    const raw = 'h1\x1fp1 p2\x1fMerge x\x1f\x1e\nh2\x1f\x1ffeat: a\x1fcorpo\x1e\n';
    expect(parseLog(raw)).toEqual([
      { hash: 'h1', parents: ['p1', 'p2'], subject: 'Merge x', body: '' },
      { hash: 'h2', parents: [], subject: 'feat: a', body: 'corpo' },
    ]);
    expect(parseLog('')).toEqual([]);
  });
});
