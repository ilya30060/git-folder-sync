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

const gradleProperties = path.join(gen, 'gradle.properties');
let props = fs.existsSync(gradleProperties) ? fs.readFileSync(gradleProperties, 'utf8') : '';
const settings = [
  'org.gradle.daemon=false',
  'org.gradle.workers.max=2',
  'org.gradle.jvmargs=-Xmx3g -Dfile.encoding=UTF-8',
  'kotlin.incremental=false',
  'kotlin.compiler.execution.strategy=in-process'
];
for (const setting of settings) {
  const key = setting.split('=')[0];
  const re = new RegExp(`^${key.replace('.', '\\.')}=.*$`, 'm');
  if (re.test(props)) props = props.replace(re, setting);
  else props += (props.endsWith('\n') || props.length === 0 ? '' : '\n') + setting + '\n';
}
fs.writeFileSync(gradleProperties, props);

console.log('Android SAF plugin and Gradle build settings prepared.');
