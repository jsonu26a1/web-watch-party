guide to watch
==============

in general, there are many different ways to setup a watch party, and it depends on a number of factors;

- where the media is hosted;
  - a video on a streaming service like Netflix, Hulu, Prime, etc
  - a video/content hosting service like YouTube, Google Drive, etc
  - a file stored locally on a participants device
- technical familiarity/comfort level of each participant;
  - can click a link to visit a web app in their browser
  - can install a web extension on their browser
  - can run a desktop app to use NAT-PMP/PCP to host a TURN server

here are a few specific types of setups to consider;
1. a host plays a video on their device, and shares their screen to the other participants (over a calling/messaging app, ie discord)
   - pros: easy, accessible, simple
   - cons: bad watching experience (live stream of screen with bad compression and dropped frames)
2. each participant accesses the video via a service (hosting or streaming) on their device, and playback is synchronized via a web extension
   - pros: good watching experience
   - cons: each participant must be able to access service (consider: paid accounts, geoblocking), requires web extension
3. a host streams a local file to each participant
   - pros:
     - (potentially) good watching experience
     - (most participants) only require a web app
     - flexibility/freedom from service providers
   - cons:
     - depends on peer-to-peer connections (network bandwidth concerns)
     - most likely requires access to or hosting a TURN server because of NATs (home routers)
     - desired media must be located and downloaded ahead of time
