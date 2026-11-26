// Copies the marks from assets/ into static/img before every build, so the
// site uses the same files as the README and never keeps its own copy.
import {copyFileSync, mkdirSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const target = resolve(here, '../static/img');

mkdirSync(target, {recursive: true});

for (const name of ['nib-mark.svg', 'nib-appicon.svg', 'nib-cli.svg']) {
  copyFileSync(resolve(root, 'assets', name), resolve(target, name));
}

// The navbar and favicon want one stable name.
copyFileSync(resolve(root, 'assets/nib-mark.svg'), resolve(target, 'mark.svg'));
