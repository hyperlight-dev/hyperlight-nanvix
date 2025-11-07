# AI-Generated Scripts Example

AI-generated code from LLMs is untrusted and should never run directly in the host environment to protect the host system from potentially malicious code. This example demonstrates running AI-generated JavaScript code safely in a Nanvix sandbox.

## Setup

1. Pack the hyperlight-nanvix package from the root:
```bash
cd ../..
npm pack
```

2. Install dependencies:
```bash
cd examples/ai-generated-scripts
npm install
```

3. Configure your OpenAI API key:
```bash
cp .env.example .env
# Edit .env and add your OPENAI_API_KEY
```

## Run

```bash
# Generate and run 1 script (default)
npm start

# Generate and run multiple scripts
npm start 5
```
