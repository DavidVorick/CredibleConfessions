// Verify.js assumes that index.js is loaded. verify.js starts off by loading
// the skylink of the document from the fragment, and then loads it into the
// text and verifier.

// performVerification will fetch the provided confession and verify it.
async function performVerification() {
  // First step is to look at the fragment and fetch the data from Skynet.
  const skylinkHash = window.location.hash
  const skylink = skylinkHash.substring(1)
  if (skylink.length !== 46) {
    setVerificationStatus("#D50000", "document hash should be 46 characters")
    return
  }

  // Fetch the skylink from Skynet.
  let skylinkData
  try {
    const response = await fetch("https://siasky.net/"+skylink)
    if (!(response.ok)) {
      setVerificationStatus("#D50000", `Unable to download data, got response code ${response.status}`)
      return
    }
    skylinkData = await response.json()
  } catch(err) {
    setVerificationStatus("#D50000", `Unable to download data: ${err}`)
    return
  }

  // Download complete, update the verification status to processing.
  setVerificationStatus("#FCD083", "Processing")

  // TODO: Need to verify the skylinkData to make sure it's well formed and
  // won't harm the app.

  // Set the message DOM so the user has something to look at.
  setMessageDOM(skylinkData)

  // Check that the signature is valid.
  let publicKeys = []
  const authors = skylinkData.authors
  for (let i = 0; i < authors.length; i++) {
    publicKeys.push(...authors[i].keys)
  }
  const isValidProofResp = await postWorkerMessage({
    method: "verify",
    publicKeys,
    message: skylinkData.message,
    proof: skylinkData.proof,
  })
  const isValidProof = isValidProofResp.isValidProof
  if (isValidProof !== "") {
    setVerificationStatus("D50000", `Cryptographic proof is invalid: ${isValidProof}`)
    return
  }

  // Download complete, update the verification status to processing.
  setVerificationStatus("#00A500", "Proof is Valid")
}

// setVerificationStatus will change the color and message of the verification
// status bar.
function setVerificationStatus(color, message) {
  document.getElementById("verificationStatusDiv").style = "background: "+color+"; width: 100%"
  document.getElementById("verificationStatusP").innerHTML = message
}

// setMessageDOM will fill out the DOM of the page with the message and the
// authors.
//
// TODO: Need to make sure this is safe... can html injection happen here?
function setMessageDOM(skylinkData) {
  // Set the read body.
  document.getElementById("readMessage").innerHTML = skylinkData.message

  // Set all of the authors. Verify their keys one at a time.
  const authors = skylinkData.authors
  for (let i = 0; i < authors.length; i++) {
    const author = authors[i]

    // Create the DOM for the avatar img.
    const avatarImg = document.createElement("img")
    avatarImg.src = author.avatarURL
    avatarImg.style = "float: left; margin: 10px; height: 100px; width: 100px"

    // Create the DOM for the author text.
    const nameDiv = document.createElement("div")
    nameDiv.style = "float: right; width: calc(100% - 120px); display: flex; justify-content: center; align-content: center; flex-direction: column"
    const nameP = document.createElement("p")
    nameP.style = "font-size: 24px; padding-left: 10px"
    nameP.innerHTML = author.username
    nameDiv.appendChild(nameP)

    // Create the DOM for the author div.
    const authorDiv = document.createElement("div")
    authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000; background: #FCD083"
    authorDiv.appendChild(avatarImg)
    authorDiv.appendChild(nameDiv)
    authorDiv.id = ("githubAuthor"+author.username)

    // Add the DOM for the new author.
    const authorsDiv = document.getElementById("authors")
    authorsDiv.appendChild(authorDiv)

    // Kick off the verification process.
    verifyAuthor(authors[i])
  }
}

// verifyAuthor will check the github API to make sure that all the keys listed
// for that author actually exist under their name on github.
async function verifyAuthor(author) {
  // Fetch the set of keys for this author from github.
  const keysAPI = `https://api.github.com/users/${author.username}/keys`

  // Fetch the div for this author.
  const authorDiv = document.getElementById("githubAuthor"+author.username)

  try {
    // Grab the author's keys from github.
    const response = await fetch(keysAPI)
    if (!(response.ok)) {
      authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000; background: #D50000"
      console.log(`got response error for author ${author.username}: ${response.status}`)
      return
    }
    const keysObj = await response.json()

    // Check that every reported key for this author is on github.
    allFound = true
    for (let i = 0; i < author.keys.length; i++) {
      let found = false
      for(let j = 0; j < keysObj.length; j++) {
        if (keysObj[j].key === author.keys[i]) {
          found = true
          break
        }
      }
      if (found === false) {
        allFound = false
        authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000; background: #D50000"
        console.log(`author key is missing from github ${author.username}: ${response.status}`)
        alert(author.keys[i])
        alert(JSON.stringify(keysObj))
      }
    }
    if (allFound === true) {
        authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000; background: #00A500"
    }
  } catch(err) {
    authorDiv.style = "width: 100%; display: flex; border-bottom: thin solid #000000; background: #D50000"
    console.log(`got response error for author ${author.username}: ${err}`)
    return
  }
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

performVerification()
