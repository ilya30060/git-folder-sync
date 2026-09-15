import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const gen = path.join(root, 'src-tauri', 'gen', 'android');
const app = path.join(gen, 'app');
const javaDir = path.join(app, 'src', 'main', 'java', 'com', 'gitfoldersync', 'saf');
const src = path.join(root, 'android-saf-plugin');

if (!fs.existsSync(app)) {
  throw new Error('Android project not initialized. Run: npm run tauri android init');
}

fs.mkdirSync(javaDir, { recursive: true });
for (const file of ['AndroidSafPlugin.kt', 'TreeSync.kt']) {
  fs.copyFileSync(path.join(src, file), path.join(javaDir, file));
}

const gradle = path.join(app, 'build.gradle.kts');
let text = fs.readFileSync(gradle, 'utf8');
const dependency = '    implementation("androidx.documentfile:documentfile:1.0.1")';
if (!text.includes('androidx.documentfile:documentfile')) {
  const marker = 'dependencies {';
  if (!text.includes(marker)) throw new Error(`Cannot find dependencies block in ${gradle}`);
  text = text.replace(marker, `${marker}\n${dependency}`);
  fs.writeFileSync(gradle, text);
}

console.log('Android SAF plugin prepared.');
