#!/usr/bin/env ts-node
import fs from 'fs';
import path from 'path';
import yaml from 'yaml';

const ROOT = process.cwd();
const YAML_DIR = path.join(ROOT, 'locales');
const OUT_DIR = path.join(ROOT, 'src', 'locales');
const SUPPORTED = ['en', 'ko', 'tr', 'pt-PT', 'de'] as const;

type _Locale = (typeof SUPPORTED)[number];

type YamlPrimitive = string | number | boolean | null;
type YamlNode = YamlPrimitive | YamlTree | YamlNode[];
type YamlTree = { [key: string]: YamlNode };

type _Bundle = Record<string, unknown>;

const ensureDir = (dir: string) => {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
};

const main = () => {
  ensureDir(OUT_DIR);

  // Process each locale's YAML file separately
  for (const locale of SUPPORTED) {
    const srcPath = path.join(YAML_DIR, `${locale}.yml`);

    if (!fs.existsSync(srcPath)) {
      console.warn(`Warning: ${srcPath} not found, skipping...`);
      continue;
    }

    const raw = fs.readFileSync(srcPath, 'utf8');
    const parsed = yaml.parse(raw) as YamlTree;

    // No need to flatten by locale anymore since each file is already single-language
    const outPath = path.join(OUT_DIR, `${locale}.json`);
    fs.writeFileSync(outPath, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8');
    console.log(`Generated ${outPath}`);
  }
};

main();
