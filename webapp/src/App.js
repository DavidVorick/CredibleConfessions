import logo from './logo.svg';
import './App.css';
import * as React from "react";

// TODO: Currently don't check for duplicate authors

// TODO: Currently no way to delete authors

// TODO: Save the message and set of authors in localstorage frequently so that
// a user doesn't lose progress if they refresh the page.

// TODO: Make the app less ugly

// WriteHeaderCard defines the header for the page where you write a
// confession. It's got a title and a link where you can learn more about the
// app.
function WriteHeaderCard() {
  return (
    <center>
      <h1>Write a Credible Confession</h1>
      <h3><a href="https://google.com" target="_blank" rel="noopener noreferrer">(how it works)</a></h3>
    </center>
  )
}

// WriteBodyCard defines the body for the page where you write a confession. It
// contains an author's section and a section for writing your confession.
function WriteBodyCard() {
  return (
    <div style={{ width: "100%", display: "flex" }}>
      <div style={{ float: "left", width: "400px", background: "#FFAAAA" }}>
        <AuthorsCard />
      </div>
      <div style={{ float: "right", width: "calc(100% - 400px)", background: "#888888" }}>
        <MessageCard />
      </div>
    </div>
  )
}

// WriteFooterCard defines the footer for the page where you write a
// confession.
function WriteFooterCard() {
  return (
    <div style={{ width: "100%", background: "#3333FF" }}>
      <center>
        <h1>Socials</h1>
      </center>
    </div>
  )
}

// AuthorsCard defines the authors section of the page where you write a
// confession. It contains one element for each author already added, plus a
// button to add more authors.
interface Author {
  platform: string;
  avatarURL: string;
  username: string;
  keys: string[];
}
// There is a set of authors that gets tracked internally to react, and a
// separate (but matching) global set of authors that is used to create the
// pubkey list that gets provided to the signing software.
const authorSet: Author[] = []
function AuthorsCard() {
  const [authorList, setAuthorList] = React.useState([])

  // addAuthor will add an author to the set of authors. It needs to be
  // provided as a prop to the AddAuthorCard component.
  const addAuthor = async () => {
    const username = document.getElementById("authorTextField").value
    const avatarURL = `https://github.com/${username}.png`
    const keysAPI = `https://api.github.com/users/${username}/keys`

    // Grab the author keys, alert an error if the keys can't be fetched.
    try {
      // Fetch the keys using the keysAPI.
      const response = await fetch(keysAPI)
      if (!(response.ok)) {
        alert(`error adding ${username}: ${response.status}`)
        return
      }

      // Parse the response.
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
          alert(keysObj[i].key.length)
          continue
        }
        keys.push(keysObj[i].key)
      }

      // Add the author to the state.
      const newAuthor = {
        platform: "GitHub",
        username,
        avatarURL,
        keys,
      }
      const newAuthorList = [...authorList]
      newAuthorList.push(newAuthor)
      setAuthorList(newAuthorList)
      authorSet.push(newAuthor)
    } catch (err) {
      alert(`error adding ${username}: ${err}`)
    }
  }

  return (
    <div style={{ width: "100%" }}>
      {authorList.map(author => (
        <AuthorCard username={author.username} avatarURL={author.avatarURL} />
      ))}
      <AddAuthorCard addAuthor={addAuthor}/>
    </div>
  )
}

// AuthorCard defines the card for a single author. It contains an avatar and a
// username.
function AuthorCard(props) {
  return (
    <div style={{ width: "100%", borderBottom: "solid #000000", display: "flex" }}>
      <div style={{ float: "left", width: "132px" }}>
        <img src={props.avatarURL} style={{ margin: "16px", height: "100px", width: "100px" }} />
      </div>
      <div style={{ float: "right", width: "calc(100% - 132px)", display: "flex", justifyContent: "center", alignContent: "center", flexDirection: "column" }}>
        <h1>{props.username}</h1>
      </div>
    </div>
  )
}

// AddAuthorCard defines the card for adding a author to the list of authors.
function AddAuthorCard(props) {
  return (
    <center>
      <input type="text" id="authorTextField" />
      <button style={{ margin: "20px"}} onClick={props.addAuthor}>Add Author</button>
    </center>
  )
}

// MessageCard defines the part of the app where you author the message.
function MessageCard() {
  // getSignInfo alerts the user with all the info they need to produce a
  // signature.
  function getSignInfo() {
    alert(JSON.stringify({
      message: document.getElementById("message").value,
      authors: authorSet,
    }))
  }

  return (
    <div style={{ width: "100%" }}>
      <div style={{ width: "100%" }}>
        <p style={{ padding: 0, paddingLeft: "30px" }}>Write your confession:</p>
      </div>
      <center>
        <div style={{ width: "100%" }}>
          <textarea id="message" style={{ width: "calc(100% - 120px)", minWidth: "240px", height: "600px", minHeight: "260px", margin: "20px" }} />
        </div>
      </center>
      <div style={{ width: "100%" }}>
        <p style={{ padding: 0, paddingLeft: "30px" }}>
          Add Your Private Key or Signature: &nbsp;
          <a onClick={getSignInfo} href="javascript:void(0)">(click here for the data to sign)</a>
        </p>
      </div>
      <center>
        <div style={{ width: "100%" }}>
          <textarea id="message" style={{ width: "calc(100% - 120px)", minWidth: "240px", height: "120px", minHeight: "80px", margin: "20px" }} />
        </div>
      </center>
      <center>
        <div style={{ width: "100%" }}>
          <button onClick={getSignInfo} style={{ margin: "20px" }}>Publish</button>
        </div>
      </center>
    </div>
  )
}

// App defines the structure of the index page.
function App() {
  document.title="Write a Credible Confession"
  return (
    <div style={{ background: "#AAAAAA", height: "100%", width: "100%" }}>
      <WriteHeaderCard />
      <WriteBodyCard />
      <WriteFooterCard />
    </div>
  )
}

export default App;
