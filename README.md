# dexlib-rs

Rust parser/writer for the Dalvik executable format.

**Currently WIP.**

### Testing

To test the library, I ripped the `classes.dex` file from YouTube's APK.
I'm obviously not going to include it in the repository, so you'll have to get it yourself from APKMirror.
Using the YouTube APK is not required, you can use any APK you want if you wish to do so.

Once you have it placed in the [`tests`](./tests/) directory, you can run the tests with:

```
cargo test
```
