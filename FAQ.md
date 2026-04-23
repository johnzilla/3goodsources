## FAQ

<details>
<summary>What is 3 Good Sources (3GS)?</summary>

3 Good Sources is a federated system for cryptographically-signed, high-quality source recommendations for AI agents.

For every topic, a human curator selects **exactly three** vetted sources. These are served via the Model Context Protocol (MCP) with PKARR-based identity and signatures, so agents can verify provenance and trust.

The deliberate limit of three sources prioritizes quality and curation over quantity.

</details>

<details>
<summary>Why only three sources per topic?</summary>

AI agents are overwhelmed by noisy, SEO-gamed search results. The strict “three sources” rule forces high-signal curation: primary documentation, authoritative references, and practical guides — no affiliate spam, no listicles.

</details>

<details>
<summary>How does federation work?</summary>

Curators run independent 3GS nodes and endorse each other via a web-of-trust model (similar to PGP).

- Agents can query a single node (`get_sources`) or the federated network (`get_federated_sources`).
- Results are tagged with trust level: `direct` (local curator) or `endorsed` (peer curator).
- Nodes periodically cache peer registries for resilience.

</details>

<details>
<summary>What is the Model Context Protocol (MCP)?</summary>

MCP is the emerging standard for giving AI agents/tools structured access to external capabilities. 3GS implements a full MCP server, allowing tools like Claude Desktop, Cursor, or custom agents to call `get_sources`, `get_federated_sources`, etc.

</details>

<details>
<summary>Can I run my own curator node?</summary>

Yes — very easily.

```bash
cargo run -- fork --endorse <existing-pubkey> --url <existing-url> --name "Your Name"

This scaffolds a new node with fresh PKARR identity, skeleton registry, and configuration. Add your own curated categories and run it.</details>

<details>
<summary>How are sources verified?</summary>

Every registry is cryptographically signed using the node’s PKARR keypair. Agents (and humans) can:Call get_provenance to see the curator’s identity
Verify signatures against the published registry
Check the public audit log for changes

</details>

<details>
<summary>Is it production-ready?</summary>

It is stable with a full MCP implementation, federation, audit logging, and Docker support. However, the ecosystem (MCP clients, adoption by agents) is still early. Suitable for experimentation and real use by privacy/AI-curious users and developers.</details>

<details>
<summary>What kinds of topics are curated?</summary>

Anything where quality matters: learning programming languages, running Bitcoin nodes, privacy tools, home automation, security practices, etc. Curators are encouraged to focus on practical, primary sources.</details>

<details>
<summary>What are next planned improvements?</summary>

More MCP tools (e.g., contribution workflow)
Better web UI / registry browser
Reputation and quality scoring across the network
Expanded community contribution system
Official MCP client integrations

</details>

<details>
<summary>How can I help?</summary>

Star the repo
Run your own node and endorse others
Submit curated categories or proposals
Build MCP clients that use 3GS
Review code, security model, or documentation

</details>

