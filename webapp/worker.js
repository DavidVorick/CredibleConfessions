// Import the wasm stuff and bind it.
importScripts("./pkg/ringsig.js");
const {prove, is_secret_key, verify} = wasm_bindgen;

// init returns a promise that resolves when the wasm bindings have completed.
async function init() {
  await wasm_bindgen("./pkg/ringsig_bg.wasm")
  return
}

// Set up the handler for when the main page sends a message.
onmessage = async function(event) {
  await init()

  // Check if the caller wants a proving operation.
  if (event.data.method === "prove") {
    const proof = prove(event.data.publicKeys, event.data.message, event.data.secretKey)
    postMessage({
      proof,
      nonce: event.data.nonce,
    })
    return
  }

  // Check if the caller wants to verify a pubkey.
  if (event.data.method === "isSecretKey") {
    const isSecretKey = is_secret_key(event.data.secretKey)
    postMessage({
      isSecretKey,
      nonce: event.data.nonce,
    })
    return
  }

  // Check whether the caller wants to know whether the input is a valid proof.
  if (event.data.method === "verify") {
    const isValidProof = verify(event.data.proof, event.data.publicKeys, event.data.message)
    postMessage({
      isValidProof,
      nonce: event.data.nonce,
    })
    return
  }
}
