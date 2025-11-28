#!/usr/bin/env ts-node
import fs from 'fs';
import path from 'path';
import yaml from 'yaml';

const ROOT = process.cwd();
const YAML_DIR = path.join(ROOT, 'locales');
const OUT_DIR = path.join(ROOT, 'src', 'locales');
const SUPPORTED = ['en', 'ko', 'tr', 'pt-PT', 'de'] as const;

type Locale = (typeof SUPPORTED)[number];

type YamlPrimitive = string | number | boolean | null;
type YamlNode = YamlPrimitive | YamlTree | YamlNode[];
type YamlTree = { [key: string]: YamlNode };

type Bundle = Record<string, unknown>;

const ensureDir = (dir: string) => {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
};

const flattenLocales = (tree: YamlTree, locale: Locale): Bundle => {
  const walk = (node: YamlNode): unknown => {
    if (node === null) return null;
    if (typeof node === 'string' || typeof node === 'number' || typeof node === 'boolean') {
      return node;
    }
    if (Array.isArray(node)) {
      return node.map((item) => walk(item));
    }
    // If node is an object
    const keys = Object.keys(node);
    const isLocaleMap = keys.length > 0 && keys.every((k) => SUPPORTED.includes(k as Locale));
    if (isLocaleMap) {
      const map = node as Record<string, unknown>;
      return map[locale];
    }
    const obj: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(node)) {
      obj[k] = walk(v as YamlNode);
    }
    return obj;
  };

  return walk(tree, {}) as Bundle;
};

const main = () => {
  ensureDir(OUT_DIR);
  const files = fs.readdirSync(YAML_DIR).filter((f) => f.endsWith('.yml') || f.endsWith('.yaml'));
  for (const file of files) {
    const srcPath = path.join(YAML_DIR, file);
    const raw = fs.readFileSync(srcPath, 'utf8');
    const parsed = yaml.parse(raw) as YamlTree;

    for (const locale of SUPPORTED) {
      const bundle = flattenLocales(parsed, locale);
      const outPath = path.join(OUT_DIR, `${locale}.json`);
      // Prettier-compatible output (2-space indent + trailing newline)
      fs.writeFileSync(outPath, `${JSON.stringify(bundle, null, 2)}\n`, 'utf8');
      console.log(`Generated ${outPath}`);
    }
  }
};

main();
