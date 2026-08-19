import * as esbuild from 'esbuild'
import { readFileSync, writeFileSync, mkdirSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'))
const pkgName = pkg.name

async function build() {
  const result = await esbuild.build({
    entryPoints: [resolve(root, 'src/client/index.ts')],
    bundle: true,
    format: 'cjs',
    platform: 'browser',
    target: 'es2020',
    external: [
      'react',
      'react/jsx-runtime',
      '@deepseek-ai/dsh-client-ui-primitives',
    ],
    jsx: 'automatic',
    jsxImportSource: 'react',
    write: false,
    sourcemap: false,
    minify: false,
    keepNames: true,
  })

  let code = result.outputFiles[0].text

  // Wrap in __ModuleLoader__.load() format
  const lines = [
    `window.__ModuleLoader__.load({ id: "${pkgName}", factory: (require) => {`,
    `  var module = { exports: {} };`,
    `  var exports = module.exports;`,
    `  Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });`,
    code,
    `  return module.exports;`,
    `}});`,
  ]

  const outDir = resolve(root, 'client')
  mkdirSync(outDir, { recursive: true })
  writeFileSync(resolve(outDir, 'client.js'), lines.join('\n'), 'utf-8')
  console.log('✓ Built client/client.js')
}

build().catch(err => {
  console.error(err)
  process.exit(1)
})