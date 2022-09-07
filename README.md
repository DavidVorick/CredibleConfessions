# Credible Confessions

Credible Confessions is an application that allows prominent members of a
community to release anonymous yet credible pieces of writing. The author is
able to create a signature that combines their public key with the public keys
of any other people that they would like to establish as potential authors.

The final signature can be verified by the general public as definitely
belonging to at least one of the potential authors, however they have no
ability to tell which person is the real author. Furthermore, none of the
potential authors have any ability to prove that it wasn't them, nor do they
have any ability to opt-out of being assigned as a potential author.

This is useful for leaking secrets or spreading gossip. For example, an execute
at a large company could write an article that exposes the misdeeds of a
corporation. They can use credible confessions to establish that the article
comes from a respected executive, without having to reveal their full identity.

## Verifying a Credible Confession

The Credible Confessions app can be used to read and verify a credible
confession. This piece of writing here is an example of a credible confession.
Verification is broken into two steps.

The first step is to verify the signature cryptographically. The app will make
sure that the writing is signed, and that the signature matches the set of
provided public keys. The status of the cryptographic signature is displayed in
the bar at the top of the screen. If the cryptographic signature is not valid,
then the writing is definitely a forgery.

The second step is to verify that the public keys belong to interesting people.
Currently, this app only supports public keys that are imported from GitHub. We
have plans to extend the app in the future to support additional sources of
public keys, such as Skynet keys, Farcaster keys, HNS keys, ENS keys, and keys
from other ecosystems as well.

The list of authors is displayed to the left. If the background of the author
is yellow, that author is still being verified. If the background of the author
is green, it means that the app was successfully able to verify that their
public key existed on Github. If the background of the author is red, it means
that the app was unable to find the author's public key on Github. This can
happen if the message is a forgery, but it can also happen if the author
deleted their public key after the signed message was published. Therefore, a
red author indicates that further investigation is needed, and that the message
should be treated with suspicion.

## Writing a Credible Confession

Writing a credible confession has three steps. The first is to write the
message, the second is to add all of the desired authors, and the final step is
to produce a signature. Writing the message is self-explanatory.

To add a potential author to the document, you write out their Github username
and click the 'Add Author' button. In the background, the app will pull their
public keys from Github and add them to the ring signature. When their username
and profile picture appear in the list of keys, you know that they have been
successfully added.

There are two ways to sign the document. The first and easiest way to sign the
document is to paste your Github ssh private key into the signature box. When
you provide your private key, the data will be signed using a ring signature
and a link to the final piece of writing will be produced. NOTE: the app
currently does not support private keys that are protected by a password.

Some people may not be comfortable pasting their ssh private key into a random
webapp. This is a reasonable concern, therefore we also provide a CLI tool that
can produce a signature. If you click the link that says '(click here for the
data to sign)', a download will start for a file that contains all of the key
information necessary to produce a signature.

You can find the repo for the CLI tool at
https://github.com/DavidVorick/CredibleConfessions/

You can build the tool by navigating to the `ringsig` directory and running the
command `cargo build --release`. This will put a `ringsig-cli` binary in the
`ringsig/target/release` folder. You can create a signature by running
`./ringsig-cli prove <message file> > proof.dat`. To see a list of advanced
options, you can run `./ringsig-cli`

Once you have created the proof, you can paste the proof into the Credible
Confessions app. The proof will get verified to ensure that no mistakes were
made, and if the proof is valid a link to the final piece of writing will be
produced.

## Future Work

The authors of this software are working on an update that will transition from
using AOS signatures to using bulletproofs. The use of bulletproofs will allow
us to mix many different types of public keys into a single signature.
Ambitiously, we could produce ring signatures that contain ed25519 keys, RSA
keys, and ECDSA keys all in the same ring. There is no ETA for this update
currently.

The current software only supports using ssh keys for signing. An easy first
extension would be to support other forms of ed25519 keys as well. Pull
requests welcome!

Another form of extension that would be highly welcome is the addition of more
sources for keys than just github. The AddAuthor section of the page would need
to be extended with a dropdown to select which source a key is being pulled
from, and the verify page would need to be extended to support more sources
than github. Ideas for sources would include ENS, HNS, Farcaster, and other
ecosystems that have lots of keys. Pull requests welcome!

There is also significant opportunity to upgrade the frontend of the
application. At this time, we will not be merging any changes that introduce
frameworks or javascript dependencies. If you would like to rebuild this
project to use something like React and/or Typescript or even npm, please fork
the project rather than make a pull request. The code is fully open source, and
forks are highly encouraged.
