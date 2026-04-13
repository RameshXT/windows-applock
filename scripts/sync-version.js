import { readFileSync, writeFileSync } from 'fs';

const packageJsonPath = 'package.json';
const tauriConfPath = 'src-tauri/tauri.conf.json';
const cargoTomlPath = 'src-tauri/Cargo.toml';

const pkg = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
const currentVersion = pkg.version;

console.log(`Current version: ${currentVersion}`);

let newVersion = process.argv[2];

if (!newVersion || ['patch', 'minor', 'major'].includes(newVersion)) {
  const parts = currentVersion.split('.').map(Number);
  const type = newVersion || 'patch';
  
  if (type === 'major') parts[0]++;
  if (type === 'minor') parts[1]++;
  if (type === 'patch') parts[2]++;
  
  if (type === 'major') { parts[1] = 0; parts[2] = 0; }
  if (type === 'minor') { parts[2] = 0; }
  
  newVersion = parts.join('.');
  console.log(`Bumping ${type} version...`);
}

console.log(`Target version: ${newVersion}`);

const updateJson = (path, version) => {
  const content = JSON.parse(readFileSync(path, 'utf8'));
  content.version = version;
  writeFileSync(path, JSON.stringify(content, null, 2) + '\n');
  console.log(`✅ Updated ${path}`);
};

const updateCargo = (path, version) => {
  let content = readFileSync(path, 'utf8');
  content = content.replace(/^version = ".*"/m, `version = "${version}"`);
  writeFileSync(path, content);
  console.log(`✅ Updated ${path}`);
};

try {
  updateJson(packageJsonPath, newVersion);
  updateJson(tauriConfPath, newVersion);
  updateCargo(cargoTomlPath, newVersion);
  console.log('\n🚀 ALL FILES SYNCED SUCCESSFULLY!');
  console.log(`Next step: git add . && git commit -m "release: ${newVersion}" && git tag v${newVersion} && git push origin --tags`);
} catch (err) {
  console.error('❌ Error syncing versions:', err.message);
  process.exit(1);
}
