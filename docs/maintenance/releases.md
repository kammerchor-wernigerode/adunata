# Releases

Every push to `main` that passes CI starts the workflow [`publish`](../../.github/workflows/publish.yaml). It publishes the image of that commit as `sha-<commit>` and moves `nightly` to the newest commit on `main` that has an image. A release promotes such a commit. The workflow [`release`](../../.github/workflows/release.yaml) builds nothing. It takes the archives and the image that CI verified for the commit.

1. Raise `version` in `server/Cargo.toml` and `client/package.json`, for example to `0.2.0`, and run `cargo update --workspace` in `server/` so that `server/Cargo.lock` follows. Regenerate the API document and the client code as [API contract](../../CONTRIBUTING.md#api-contract) describes, because the document carries the version.
2. Merge the change through a pull request. Wait until CI and `publish` passed for its commit on `main`.
3. Try the release without publishing anything. Under Actions, `Release`, Run workflow, enter the tag and that merge commit. The run checks the conditions and the archives and stops before it writes anything.
4. Tag the commit on `main` that CI ran for, which is the merge commit of the pull request, not the commit inside it. Push the tag:

    ```sh
    git tag v0.2.0 <commit>
    git push origin v0.2.0
    ```

The workflow stops with a message when the tag is not on `main`, when the tag does not equal the version of all three files, when the commit does not raise the version above its first parent, when `sha-<commit>` does not exist or when CI no longer keeps the archives of the commit. CI keeps them for 30 days. After that, a new commit that raises the version is needed. A version with a pre-release part, such as `0.2.0-rc.1`, becomes a pre-release and leaves `latest` where it is. A failed run can run again within a day, because each step accepts what an earlier attempt left and the archives between its jobs last a day. Later, run all its jobs again. The same holds for `publish`, because CI keeps the images for a day.
