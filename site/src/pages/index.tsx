import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import type {ReactNode} from 'react';

import styles from './index.module.css';

const claims: Array<{title: string; body: string}> = [
  {title: 'Local', body: 'One SQLite file on your disk. Nothing is uploaded.'},
  {
    title: 'Searchable',
    body: 'FTS5 over the text of every paste, matches shown as snippets.',
  },
  {
    title: 'Yours',
    body: 'MIT, no account, no relay, no telemetry, no network calls.',
  },
];

export default function Home(): ReactNode {
  return (
    <Layout
      title="Offline pastebin for your terminal"
      description="nib keeps text in one local SQLite file, reachable from the terminal and from a window. No network, no account.">
      <main className={styles.page}>
        <section className={styles.hero}>
          <img
            className={styles.mark}
            src={useBaseUrl('/img/nib-appicon.svg')}
            alt=""
            width={112}
            height={112}
          />
          <h1 className={styles.title}>Text lives here.</h1>
          <p className={styles.lead}>
            An offline pastebin for your terminal, with a window beside it.
            One store, two surfaces, nothing leaving the machine.
          </p>
          <div className={styles.actions}>
            <Link className={styles.primary} to="/docs/spec">
              Read the spec
            </Link>
            <Link
              className={styles.secondary}
              href="https://github.com/Raghav2012Code/nib">
              Source
            </Link>
          </div>
          <pre className={styles.terminal}>
            <code>
              <span className={styles.prompt}>$ </span>
              echo &quot;kubectl get pods -A&quot; | nib{'\n'}
              <span className={styles.prompt}>$ </span>
              nib search &quot;get pods&quot;{'\n'}
              4f2a kubectl get pods -A
            </code>
          </pre>
        </section>

        <section className={styles.claims}>
          {claims.map((claim) => (
            <div className={styles.claim} key={claim.title}>
              <h2>{claim.title}</h2>
              <p>{claim.body}</p>
            </div>
          ))}
        </section>

        <section className={styles.docs}>
          <h2>Docs</h2>
          <ul>
            <li>
              <Link to="/docs/spec">The spec</Link> — scope, data model, CLI
              surface, app and packaging.
            </li>
            <li>
              <Link to="/docs/plan">The plan</Link> — what each phase built and
              what it proved.
            </li>
            <li>
              <Link to="/docs/design/terminal-ui">The terminal app</Link> and{' '}
              <Link to="/docs/design/logo">the mark</Link>.
            </li>
          </ul>
        </section>
      </main>
    </Layout>
  );
}
