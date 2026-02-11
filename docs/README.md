# Welcome to HEDL

Let's talk about a problem you probably have.

You're building something that talks to an LLM. Maybe it's a RAG pipeline. Maybe it's an agent that needs context about your users. Maybe it's a chatbot that pulls from your database. Whatever it is, you're serializing data and sending it through a context window.

And you're probably using JSON.

```json
{"id": "u1", "name": "Alice", "email": "alice@company.com", "role": "admin"}
{"id": "u2", "name": "Bob", "email": "bob@company.com", "role": "user"}
{"id": "u3", "name": "Carol", "email": "carol@company.com", "role": "user"}
```

Look at that. Really look at it. The words `"id"`, `"name"`, `"email"`, `"role"` appear three times. In a file with a thousand users, they appear a thousand times. You're paying for every single one of those tokens. Not because they carry information, but because JSON doesn't know any better.

At $3 per million tokens, a 10,000-user dataset costs you roughly $15 just in repeated field names. Every request. Every day. For the privilege of saying "name" ten thousand times.

There's a better way.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email,role]
---
users:@User
 |u1,Alice,alice@company.com,admin
 |u2,Bob,bob@company.com,user
 |u3,Carol,carol@company.com,user
```

Same data. The field names appear exactly once, in the schema declaration. After that, it's pure data. No ceremony. No repetition. No waste.

This is HEDL. And this documentation will teach you everything you need to know to use it.

---

## Finding Your Way

This documentation is organized around who you are and what you're trying to do. Find yourself below and follow that path.

### You want to use HEDL in your projects

You're here to get things done. You want to write HEDL files, convert your existing data, maybe set up your editor with syntax highlighting. You don't need to know how the parser works internally.

**Your journey starts at the [User Guide](user/README.md).**

There you'll learn to write HEDL by hand (it's surprisingly pleasant once you get the hang of it), convert between formats using the CLI, validate your documents, and understand when HEDL is the right choice for your problem.

Along the way, you'll work through real examples: configuration files, API responses, knowledge graphs, data pipelines. By the end, you'll be fluent.

### You want to integrate HEDL into your application

You're a developer. You need to parse HEDL in your code, convert it to JSON for an API, or maybe generate HEDL from your database. You need APIs, not tutorials.

**Your journey starts at the [API Documentation](api/README.md).**

We have bindings for Rust (native), C/C++/Python (via FFI), JavaScript/TypeScript (via WebAssembly), and protocol servers for AI agents (MCP) and editors (LSP). Pick your language, follow the quickstart, and you'll be parsing HEDL in minutes.

### You want to contribute or understand the internals

You're curious about how HEDL actually works. Maybe you want to add a feature, fix a bug, or just understand how a parser can be both fast and zero-dependency.

**Your journey starts at the [Developer Guide](developer/README.md).**

You'll learn about the lexer and parser architecture, how the AST is structured, why we made certain design decisions, and how to navigate a codebase of 19 specialized crates. We'll walk you through adding your first feature.

### You're an architect evaluating HEDL for your system

You need to understand the trade-offs. What are the performance characteristics? How does it handle errors? What are the security considerations? Will it scale?

**Your journey starts at the [Architecture Documentation](architecture/README.md).**

We'll show you the system design, the component relationships, the parsing pipeline, and the benchmarks. You'll understand exactly what you're getting and what you're giving up.

---

## The Thirty-Second Demo

Before you dive into the documentation, let's make sure HEDL is worth your time. Open a terminal:

```bash
cargo install hedl-cli
```

Now take some JSON and convert it:

```bash
echo '{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}' | hedl from-json
```

You'll see:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
users:
 |Alice,30
 |Bob,25
```

The JSON was 62 characters. The HEDL is 47. That's 24% smaller on a tiny example. On real data with repeated structures, you'll see 50-60% reductions.

Convert it back:

```bash
echo '%V:2.0
%NULL:~
%QUOTE:"
---
users:
 |Alice,30
 |Bob,25' | hedl to-json
```

You get valid JSON back. The conversion is lossless.

That's the core value proposition: same data, fewer tokens, full fidelity. Everything else in this documentation is about getting the most out of that simple idea.

---

## What HEDL Actually Is

HEDL stands for **Hierarchical Entity Data Language**. It's a text-based format for structured data, designed specifically for the economics of LLM applications.

But that's a mouthful. Here's the simpler version:

**HEDL is what JSON would look like if it were designed today, knowing that every character costs money when you send it to an AI.**

It keeps what's good about JSON:
- Human readable
- Hierarchical (objects can contain objects)
- Explicit types (strings, numbers, booleans, null)
- Easy to parse

It fixes what's wasteful:
- Repeated keys become schemas declared once
- Quotes around simple strings are optional
- Curly braces and colons are minimized
- Lists of similar items become compact rows

And it adds things JSON can't do:
- Type-safe references between entities (`@User:alice`)
- Schema validation at parse time
- Deterministic canonical formatting
- Count metadata for LLM comprehension

---

## The Documentation Map

Here's everything, laid out so you can find what you need:

```
docs/
│
├── user/                      # FOR USERS OF HEDL
│   ├── README.md              # Start here
│   ├── getting-started.md     # Your first HEDL document
│   ├── cli-guide.md           # Mastering the command line
│   ├── formats.md             # Converting to/from other formats
│   ├── examples.md            # Real-world patterns
│   ├── concepts/              # Deep dives on core ideas
│   ├── tutorials/             # Step-by-step walkthroughs
│   └── faq.md                 # Common questions answered
│
├── api/                       # FOR DEVELOPERS INTEGRATING HEDL
│   ├── README.md              # Start here
│   ├── rust-api.md            # Native Rust
│   ├── ffi-api.md             # C/C++/Python
│   ├── wasm-api.md            # JavaScript/TypeScript
│   ├── mcp-api.md             # AI agent integration
│   ├── lsp-api.md             # Editor integration
│   └── sdk/                   # Language-specific guides
│
├── developer/                 # FOR CONTRIBUTORS TO HEDL
│   ├── README.md              # Start here
│   ├── architecture.md        # How it's built
│   ├── contributing.md        # How to contribute
│   └── tutorials/             # Building features
│
└── architecture/              # FOR ARCHITECTS EVALUATING HEDL
    ├── README.md              # Start here
    ├── performance.md         # Benchmarks and characteristics
    ├── components/            # Component deep-dives
    └── diagrams/              # Visual documentation
```

---

## Getting Help

Stuck on something? Here's how to get unstuck:

**Check the FAQ first.** Your question has probably been asked before. The [FAQ](user/faq.md) covers the common ones.

**Run `hedl --help`.** Every command documents itself. `hedl validate --help` tells you exactly what flags are available.

**Search the GitHub issues.** Someone might have encountered your problem already. The [issue tracker](https://github.com/dweve-ai/hedl/issues) is public.

**Ask in Discussions.** For questions that aren't bugs, [GitHub Discussions](https://github.com/dweve-ai/hedl/discussions) is the place.

**Join the Discord.** For real-time help and community chat, [join us](https://discord.gg/dweve).

---

## Ready?

Pick your path and let's go:

| I want to... | Start here |
|--------------|------------|
| Learn to use HEDL | [User Guide](user/README.md) |
| Integrate HEDL into code | [API Documentation](api/README.md) |
| Contribute to HEDL | [Developer Guide](developer/README.md) |
| Evaluate HEDL for my system | [Architecture Docs](architecture/README.md) |

Or if you're impatient: [Getting Started](user/getting-started.md) will have you writing HEDL in five minutes.
