import * as React from "react"

// PageHeader defines the banner that goes at the top of the page.
function PageHeader(props) {
  return (
      <center>
        <h1 style={{ padding: "4px", paddingTop: "30px" }}>Write a Confession</h1>
        <h3 style={{ paddingBottom: "30px" }}><a href="https://google.com" target="_blank" rel="noopener noreferrer">(how it works)</a></h3>
      </center>
  );
}

// IndexPage defines the whole structure of the index page.
const IndexPage = () => {
  return (
    <body style={{ border: "none", margin: 0, padding: 0, height: "100%", width: "100%", position: "relative", background: "#777777" }}>
      <h1>Hi</h1>
    </body>
  );
};

export default IndexPage

/*
// PageBody defines the body that splits up the authors and the post body.
function PageBody(props) {
  return (
    <div style={{ width: "100%", position: "relative", border: "thick solid #000000" }}>
      <Authors />
      <WritePost />
    </div>
  );
};

// PageFooter defines the banner that goes at the bottom of the page.
function PageFooter(props) {
  return (
    <div style={{ backgroundColor: "#DDDDDD" }}>
        <h3 style={{ paddingBottom: "30px" }}>this is a footer</h3>
     </div>
  );
}

// Authors defines the element that 
function Authors(props) {
  return (
    <div style={{ float: "left", width: "35%", height: "100px", backgroundColor: "#999999", position: "relative" }}>
    </div>
  );
};

function WritePost(props) {
  return (
    <div style={{ float: "right", width: "65%", backgroundColor: "#777777" }}>
      <center>
        <div style={{ width: "100%" }}>
          <textarea style={{ minHeight: "160px", minWidth: "120px", marginTop: "40px", marginBottom: "0px", width: "80%" }} />
        </div>
        <button style={{ margin: "20px" }}>
          Submit
        </button>
      </center>
    </div>
  );
};
*/

