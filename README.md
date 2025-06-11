# MSG: a flavorful desktop e6 client

TODO:
- [X] add way to purge cache (but keep favorited posts)
- [X] figure out AV1 detection/support
- [X] more video controls (seeking, volume)
- [X] look into `sample` images for pre-caching? may be faster than fullsize
  - [X] make it and originals toggleable, show original by default
  - [ ] should probably refactor image/file saving logic since it's kinda duplicated
- [X] have search result store keep 'load more' posts
- [ ] *mayyyyybe* split `PostStoreData` into multiple files? would have to benchmark to make sure

## Known Issues
- [Seeking on VP9 videos is currently broken](https://discourse.gstreamer.org/t/vp9-seeking-broken-in-gstreamer-1-26/4647/4). An "open file" button has been added as a workaround - this opens it in your native media player via [`open`](https://crates.io/crates/open).
