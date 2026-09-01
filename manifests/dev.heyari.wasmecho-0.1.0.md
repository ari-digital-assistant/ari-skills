---
name: wasm-echo
description: Tiny WASM smoke-test skill. Returns a fixed greeting from inside its sandboxed module. Use only for testing the WASM loader.
license: MIT
metadata:
  ari:
    id: dev.heyari.wasmecho
    version: "0.1.0"
    author: Ari core team
    engine: ">=0.1"
    capabilities: []
    languages: [en, it]
    specificity: high
    matching:
      patterns:
        - keywords: [wasm, echo]
          weight: 0.95
    # NB: `echo` alone isn't enough — the pattern needs both words. Drop
    # `wasm` and the built-in `open` claims anything starting "run …",
    # and `greeting` claims anything containing "hello"; the router would
    # never see either, so both stay out.
    examples:
      - text: "Please test the WASM echo"
        weight: 0.95
      - text: "Test the WASM echo for me"
        weight: 0.95
      - text: "Give the WASM echo a quick test"
        weight: 0.95
      - text: "Do a quick WASM echo test"
        weight: 0.95
      - text: "Perform a WASM echo test"
        weight: 0.95
      - text: "Start a WASM echo test"
        weight: 0.95
      - text: "Run a WASM echo test"
        weight: 0.95
      - text: "Try the WASM echo test"
        weight: 0.95
      - text: "Check the WASM echo test"
        weight: 0.95
      - text: "Trigger the WASM echo test"
        weight: 0.95
      - text: "Invoke the WASM echo test"
        weight: 0.95
      - text: "Execute the WASM echo test"
        weight: 0.95
      - text: "Launch the WASM echo test"
        weight: 0.95
      - text: "Fire off the WASM echo test"
        weight: 0.95
      - text: "Put the WASM echo through a test"
        weight: 0.95
      - text: "Can you test the WASM echo?"
        weight: 0.95
      - text: "Could you test the WASM echo?"
        weight: 0.95
      - text: "Would you test the WASM echo for me?"
        weight: 0.95
      - text: "Will you run the WASM echo test?"
        weight: 0.95
      - text: "Can I get a WASM echo test?"
        weight: 0.95
      - text: "Could I get a quick WASM echo test?"
        weight: 0.95
      - text: "How about a WASM echo test?"
        weight: 0.95
      - text: "Let's test the WASM echo"
        weight: 0.95
      - text: "I'd like to test the WASM echo"
        weight: 0.95
      - text: "I want to try the WASM echo"
        weight: 0.95
      - text: "I need to check the WASM echo"
        weight: 0.95
      - text: "Time for a WASM echo test"
        weight: 0.95
      - text: "Give me the WASM echo response"
        weight: 0.95
      - text: "Show me the WASM echo response"
        weight: 0.95
      - text: "Return the WASM echo greeting"
        weight: 0.95
      - text: "Get the greeting from the WASM echo skill"
        weight: 0.95
      - text: "Ask the WASM echo skill for its greeting"
        weight: 0.95
      - text: "Have the WASM echo skill return its greeting"
        weight: 0.95
      - text: "Make the WASM echo module respond"
        weight: 0.95
      - text: "Get a response from the WASM echo module"
        weight: 0.95
      - text: "Let me hear from the WASM echo module"
        weight: 0.95
      - text: "What does the WASM echo skill return?"
        weight: 0.95
      - text: "What response comes back from the WASM echo?"
        weight: 0.95
      - text: "Can you get the WASM echo response?"
        weight: 0.95
      - text: "Could you show me the WASM echo output?"
        weight: 0.95
      - text: "I want to see the WASM echo output"
        weight: 0.95
      - text: "Show me what the WASM echo returns"
        weight: 0.95
      - text: "Tell me what the WASM echo module says"
        weight: 0.95
      - text: "Let's see what the WASM echo module says"
        weight: 0.95
      - text: "I'd like the greeting from the WASM echo module"
        weight: 0.95
      - text: "Fetch the greeting from the WASM echo module"
        weight: 0.95
      - text: "Request the greeting from the WASM echo module"
        weight: 0.95
      - text: "Call the WASM echo skill"
        weight: 0.95
      - text: "Invoke the WASM echo skill"
        weight: 0.95
      - text: "Execute the WASM echo skill"
        weight: 0.95
      - text: "Trigger the WASM echo skill"
        weight: 0.95
      - text: "Run the WASM echo skill now"
        weight: 0.95
      - text: "Start the WASM echo skill"
        weight: 0.95
      - text: "Try the WASM echo skill"
        weight: 0.95
      - text: "Call into the WASM echo module"
        weight: 0.95
      - text: "Invoke the WASM echo module"
        weight: 0.95
      - text: "Execute the WASM echo module"
        weight: 0.95
      - text: "Trigger the WASM echo module"
        weight: 0.95
      - text: "Run the WASM echo module"
        weight: 0.95
      - text: "Start the WASM echo module"
        weight: 0.95
      - text: "Can you call the WASM echo skill?"
        weight: 0.95
      - text: "Could you invoke the WASM echo module?"
        weight: 0.95
      - text: "Would you trigger the WASM echo skill?"
        weight: 0.95
      - text: "Please execute the WASM echo module"
        weight: 0.95
      - text: "Go ahead and run the WASM echo skill"
        weight: 0.95
      - text: "Go ahead and call the WASM echo module"
        weight: 0.95
      - text: "I want to invoke the WASM echo module"
        weight: 0.95
      - text: "I'd like to run the WASM echo skill"
        weight: 0.95
      - text: "I need the WASM echo module executed"
        weight: 0.95
      - text: "The WASM echo skill needs a quick run"
        weight: 0.95
      - text: "Test the WASM loader with the echo module"
        weight: 0.95
      - text: "Check the WASM loader using the echo skill"
        weight: 0.95
      - text: "Verify the WASM loader with the echo test"
        weight: 0.95
      - text: "Exercise the WASM loader with its echo module"
        weight: 0.95
      - text: "Try the echo module in the WASM loader"
        weight: 0.95
      - text: "Run the echo module through the WASM loader"
        weight: 0.95
      - text: "Put the WASM loader through an echo test"
        weight: 0.95
      - text: "Give the WASM loader an echo test"
        weight: 0.95
      - text: "Do an echo test on the WASM loader"
        weight: 0.95
      - text: "Perform the loader's WASM echo test"
        weight: 0.95
      - text: "Can you test the WASM loader with echo?"
        weight: 0.95
      - text: "Could you check the WASM loader using echo?"
        weight: 0.95
      - text: "Would you verify the WASM loader with the echo module?"
        weight: 0.95
      - text: "Please exercise the WASM loader with echo"
        weight: 0.95
      - text: "I want to test the WASM loader with its echo skill"
        weight: 0.95
      - text: "I'd like to check the WASM loader's echo module"
        weight: 0.95
      - text: "I need a WASM loader echo test"
        weight: 0.95
      - text: "Let's check the WASM loader with the echo module"
        weight: 0.95
      - text: "Let's see if the WASM loader can return the echo greeting"
        weight: 0.95
      - text: "See whether the WASM loader returns the echo response"
        weight: 0.95
      - text: "Check whether the WASM echo module loads"
        weight: 0.95
      - text: "Verify that the WASM echo skill loads"
        weight: 0.95
      - text: "Test whether the WASM echo module responds"
        weight: 0.95
      - text: "Check that the WASM echo skill responds"
        weight: 0.95
      - text: "Verify that the WASM echo module runs"
        weight: 0.95
      - text: "Make sure the WASM echo skill works"
        weight: 0.95
      - text: "Confirm that the WASM echo module works"
        weight: 0.95
      - text: "See if the WASM echo skill is working"
        weight: 0.95
      - text: "Can you check whether the WASM echo module works?"
        weight: 0.95
      - text: "Could you make sure the WASM echo skill runs?"
        weight: 0.95
      - text: "Is the WASM echo module working?"
        weight: 0.95
      - text: "Does the WASM echo skill respond?"
        weight: 0.95
      - text: "Can the WASM echo module return its greeting?"
        weight: 0.95
      - text: "Will the WASM echo skill load?"
        weight: 0.95
      - text: "Does the WASM loader pass the echo test?"
        weight: 0.95
      - text: "Is the WASM loader working with the echo module?"
        weight: 0.95
      - text: "Can you confirm the WASM loader works with echo?"
        weight: 0.95
      - text: "Could you verify the WASM echo module is available?"
        weight: 0.95
      - text: "I want to know if the WASM echo skill works"
        weight: 0.95
      - text: "I'd like to confirm the WASM echo module runs"
        weight: 0.95
      - text: "Check the sandboxed WASM echo module"
        weight: 0.95
      - text: "Test the sandboxed WASM echo skill"
        weight: 0.95
      - text: "Run the sandboxed WASM echo test"
        weight: 0.95
      - text: "Invoke the sandboxed WASM echo module"
        weight: 0.95
      - text: "Get a greeting from the sandboxed WASM echo module"
        weight: 0.95
      - text: "Can you test the sandboxed WASM echo module?"
        weight: 0.95
      - text: "Verify the sandboxed WASM echo response"
        weight: 0.95
      - text: "See if the sandboxed WASM echo module responds"
        weight: 0.95
      - text: "I'd like to try the sandboxed WASM echo skill"
        weight: 0.95
      - text: "Please call the sandboxed WASM echo module"
        weight: 0.95
      - text: "Do the WASM echo smoke test"
        weight: 0.95
      - text: "Run the WASM echo smoke test"
        weight: 0.95
      - text: "Start the WASM echo smoke test"
        weight: 0.95
      - text: "Perform the WASM echo smoke test"
        weight: 0.95
      - text: "Trigger a WASM echo smoke test"
        weight: 0.95
      - text: "Give the WASM echo module a smoke test"
        weight: 0.95
      - text: "Smoke-test the WASM echo module"
        weight: 0.95
      - text: "Can you run the WASM echo smoke test?"
        weight: 0.95
      - text: "Could you smoke-test the WASM echo module?"
        weight: 0.95
      - text: "I need a quick WASM echo smoke test"
        weight: 0.95
      - text: "Let's do a WASM echo smoke test"
        weight: 0.95
      - text: "It's time to smoke-test the WASM echo module"
        weight: 0.95
      - text: "Check the WASM echo integration"
        weight: 0.95
      - text: "Test the WASM echo integration"
        weight: 0.95
      - text: "Verify the WASM echo integration"
        weight: 0.95
      - text: "Exercise the WASM echo integration"
        weight: 0.95
      - text: "Can you check the WASM echo integration?"
        weight: 0.95
      - text: "Please test the WASM echo sandbox"
        weight: 0.95
      - text: "Verify the WASM echo sandbox"
        weight: 0.95
      - text: "Try the WASM echo sandbox"
        weight: 0.95
      - text: "Can you run the WASM echo sandbox test?"
        weight: 0.95
      - text: "Check the echo skill inside the WASM sandbox"
        weight: 0.95
      - text: "Test the echo module inside the WASM sandbox"
        weight: 0.95
      - text: "Get the fixed greeting from the WASM module"
        weight: 0.95
      - text: "Return the fixed greeting from the WASM echo module"
        weight: 0.95
      - text: "Show the fixed greeting from the WASM echo skill"
        weight: 0.95
      - text: "Can you retrieve the greeting from the WASM echo module?"
        weight: 0.95
      - text: "Could you ask the WASM echo module to respond?"
        weight: 0.95
      - text: "I could use a quick check of the WASM echo module"
        weight: 0.95
      - text: "A quick WASM echo test would be helpful"
        weight: 0.95
      - text: "echo test"
        weight: 0.95
      - text: "test the wasm loader"
        weight: 0.95
      - text: "run the wasm echo skill"
        weight: 0.95
      - text: "greeting from the wasm module"
        weight: 0.95
    wasm:
      module: skill.wasm
      memory_limit_mb: 1
---

# WASM Echo

Reference skill that exists purely to prove the WASM loader works end to end.
A minimal Rust SDK module (`src/lib.rs`, built via `build.sh`) exporting the
ABI v1 surface (`memory`, `ari_alloc`, `score`, `execute`). It returns the
`greeting` string resolved per-locale from `strings/{locale}.json` via
`ari::t()` — the canonical example of WASM-skill output localization
("wasm hello" in English, "ciao da wasm" in Italian).

## Example utterance

- "wasm echo" — the only keyword trigger, and both words are required.
  "echo" alone collides with other skills, and "wasm hello" is this skill's
  *output*, not a way to reach it: `greeting` claims anything containing
  "hello" and answers instead. The other utterances under `examples:` are
  there for the router tier and only work with a router model loaded.
