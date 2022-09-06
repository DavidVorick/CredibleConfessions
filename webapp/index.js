// Establish a global array with the full list of authors.
const authors = []

// clearAddAuthorErr will clear the error in the addAuthor section of the page.
function clearAddAuthorErr() {
  document.getElementById("addAuthorError").innerHTML = ""
}

// setAddAuthorErr will change the DOM of the addAuthor error to have the
// provided message.
function setAddAuthorErr(message) {
  document.getElementById("addAuthorError").innerHTML = message
}

// isDuplicateAuthor returns true if the provided author is a duplicate, false
// otherwise.
function isDuplicateAuthor(newAuthor) {
  for (let i = 0; i < authors.length; i++) {
    if (authors[i].username === newAuthor.username) {
      return true
    }
  }
  return false
}

// addAuthor is the function that adds an author to the state. It'll take the
// username and find all the author details, including collecting the public
// keys and updating the DOM to show the new author.
async function addAuthor() {
  // Establish the key variables for the function.
  const username = document.getElementById("addAuthorText").value
  const avatarURL = `https://github.com/${username}.png`
  const keysAPI = `https://api.github.com/users/${username}/keys`

  const newAuthor = {
    platform: "GitHub",
    username,
    avatarURL,
  }

  // Check if this author was already added. We check before the network
  // request to be quick.
  if (isDuplicateAuthor(newAuthor)) {
    setAddAuthorErr(`author has already been added`)
    return
  }

  // Grab the author keys, updating the error DOM if there's an error.
  try {
    // Make the network call to get the user's pubkeys.
    const response = await fetch(keysAPI)
    if (!(response.ok)) {
      setAddAuthorErr(`error adding ${username}: ${response.status}`)
      return
    }

    // Parse the keys from the response.
    const keysObj = await response.json()
    const keys = []
    for (let i = 0; i < keysObj.length; i++) {
      if (typeof keysObj[i].key !== "string") {
        continue
      }
      if (!(keysObj[i].key.startsWith("ssh-ed25519 "))) {
        continue
      }
      if (keysObj[i].key.length !== 80) {
        continue
      }
      keys.push(keysObj[i].key)
    }
    // Check that we got at least one usable key.
    if (keys.length === 0) {
      setAddAuthorErr(`no valid keys found`)
      return
    }
    newAuthor.keys = keys

    // We have to check if this author is a duplicate a second time because we
    // did some async work and there could have been a race condition if the
    // author was added multiple times before the network requests completed.
    if (isDuplicateAuthor(newAuthor)) {
      setAddAuthorErr(`author has already been added`)
      return
    }

    // Add the author to the set of authors.
    authors.push(newAuthor)

    // Create the DOM for the avatar img.
    const avatarImg = document.createElement("img")
    avatarImg.src = avatarURL
    avatarImg.style = "float: left; margin: 10px; height: 100px; width: 100px"

    // Create the DOM for the author text.
    const nameDiv = document.createElement("div")
    nameDiv.style = "float: right; width: calc(100% - 120px); display: flex; justify-content: center; align-content: center; flex-direction: column"
    const nameP = document.createElement("p")
    nameP.style = "font-size: 24px; padding-left: 10px"
    nameP.innerHTML = username
    nameDiv.appendChild(nameP)

    // Create the DOM for the author div.
    const authorDiv = document.createElement("div")
    authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000"
    authorDiv.appendChild(avatarImg)
    authorDiv.appendChild(nameDiv)

    // Add the DOM for the new author.
    const authorsDiv = document.getElementById("authors")
    authorsDiv.appendChild(authorDiv)

    // Operation successful, so we can clear the error now.
    clearAddAuthorErr()
  } catch(err) {
    setAddAuthorErr(err)
    return
  }
}

// publishMessage will read the state of the confession, sign it, and then
// publish it to Skynet.
async function publishMessage() {
  // Check whether the user has placed in a secret key or an already valid
  // proof.
  const sigOrKey = document.getElementById("sigOrKey").value
  const worker = await getWorker
  const isSecKeyResp = await postWorkerMessage({
    method: "isSecretKey",
    secretKey: sigOrKey,
  })

  // Get the message and set of public keys.
  const message = document.getElementById("message").value
  const publicKeys = []
  for (let i = 0; i < authors.length; i++) {
    const authorKeys = authors[i].keys
    for (let j = 0; j < authorKeys.length; j++) {
      publicKeys.push(authorKeys[j])
    }
  }

  // We have a secret key. Send all the remaining data to the prover and get a
  // proof.
  let proof
  if (isSecKeyResp.isSecretKey === true) {
    proof = await postWorkerMessage({
      method: "prove",
      publicKeys,
      message,
      secretKey: sigOrKey,
    })
  } else {
    proof = sigOrKey
  }

  alert(proof)
  alert(publicKeys)
  alert(message)

  // Double check that the proof is correct.
  const isValidProof = await postWorkerMessage({
    method: "verify",
    proof,
    publicKeys,
    message,
  })

  // TODO: Publish the final result somewhere (skynet probably)
  alert("proof:", isValidProof)
}

// Establish the promise that will launch the webworker. We launch the worker
// in a promise so that there's minimal delay when starting the app. When we
// want to use the worker in a function, we need to grab it from the promise.
const getWorker = new Promise((resolve) => {
  const worker = new Worker("worker.js")
  worker.onmessage = handleWorkerMessage
  resolve(worker)
})

// postWorkerMessage is an abstraction around worker communications to make
// simple query-response interactions painless.
let workerNonce = 0
let activeQueries = {}
async function postWorkerMessage(messageData) {
  // Get a unique nonce for this message.
  const nonce = workerNonce
  workerNonce += 1
  messageData.nonce = nonce

  // Send the message to the worker with the nonce.
  const worker = await getWorker
  worker.postMessage(messageData)

  // Craft the promise that will be resolved by handleWorkerMessage when a
  // response is received.
  return await new Promise((resolve) => {
    activeQueries[nonce] = resolve
  })
}

// handleWorkerMessage will process any messages coming from the webworker.
function handleWorkerMessage(event) {
  const nonce = event.data.nonce
  activeQueries[nonce](event.data)
  delete activeQueries[nonce]
}
