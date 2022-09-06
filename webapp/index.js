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
