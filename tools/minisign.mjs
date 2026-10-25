// Minisign, as catalogue.rs reads it (2.2.0, for the site): a prehashed
// signature (ED, BLAKE2b-512) and its trusted comment, both checked. Throws
// on anything else.

import { createHash, createPublicKey, verify } from 'node:crypto';

const ED25519_SPKI = Buffer.from('302a300506032b6570032100', 'hex');

/** The lines of a minisign file, raw or wrapped in base64 as `tauri signer` writes it. */
function minisignLines(text) {
  let t = text.trim();
  if (!t.includes('untrusted comment')) t = Buffer.from(t, 'base64').toString('utf8').trim();
  return t.split(/\r?\n/);
}

export function verifyMinisign(pubText, data, sigText) {
  const pub = Buffer.from(minisignLines(pubText)[1] || '', 'base64');
  const lines = minisignLines(sigText);
  const sig = Buffer.from(lines[1] || '', 'base64');
  const trusted = (lines[2] || '').match(/^trusted comment: (.*)$/)?.[1];
  const global = Buffer.from(lines[3] || '', 'base64');
  if (pub.length !== 42 || pub.subarray(0, 2).toString() !== 'Ed') throw new Error('cle publique illisible');
  if (sig.length !== 74 || sig.subarray(0, 2).toString() !== 'ED') throw new Error('signature illisible ou non pre-hachee');
  if (!pub.subarray(2, 10).equals(sig.subarray(2, 10))) throw new Error("signature faite avec une autre cle");
  const key = createPublicKey({ key: Buffer.concat([ED25519_SPKI, pub.subarray(10)]), format: 'der', type: 'spki' });
  const hash = createHash('blake2b512').update(data).digest();
  if (!verify(null, hash, key, sig.subarray(10))) throw new Error('signature invalide');
  if (trusted === undefined || !verify(null, Buffer.concat([sig.subarray(10), Buffer.from(trusted)]), key, global)) {
    throw new Error('commentaire de confiance invalide');
  }
}
