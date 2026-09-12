export function App() {
  return (
    <main className="readiness" aria-labelledby="winds-title">
      <p className="eyebrow">Winds Desktop</p>
      <h1 id="winds-title">Desktop shell ready.</h1>
      <p>
        This inert surface has no runtime, terminal, filesystem, Git, or provider authority.
      </p>
      <dl>
        <div>
          <dt>Authority</dt>
          <dd>Rust-owned · no renderer commands</dd>
        </div>
        <div>
          <dt>Network</dt>
          <dd>Bundled local content only</dd>
        </div>
      </dl>
    </main>
  );
}
