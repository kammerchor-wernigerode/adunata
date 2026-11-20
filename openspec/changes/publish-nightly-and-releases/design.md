# Design

## Context

- After `verify-both-architectures`, every run of the workflow `CI` builds and checks both architectures and keeps the artifacts `release-<system>` and `image-<system>`. The release binary of a push to `main` lives 30 days, images one day.
- `release.yaml` runs on tags `v*` on any commit. It runs `nix flake check`, builds the binary and the image a second time for `x86_64-linux` only and packs the archive in a shell step that no check covers. It pushes `:<tag>` and `:latest` for every tag, including pre-releases, before it creates the GitHub release, and passes the token on the command line. It creates notes with `--generate-notes`. No tag and no release exist yet.
- The version is `0.1.0` in `server/Cargo.toml` and `server/Cargo.lock`, a copy in `flake.nix` and `0.0.0` in `client/package.json`. `client/pnpm-lock.yaml` holds no version of the project itself. Nothing compares them.
- The image is a Docker archive from `dockerTools.buildLayeredImage`. The package `adunata` does not exist on ghcr.io yet. GitHub creates every package as private and offers no API to change that. A push with `GITHUB_TOKEN` from a workflow links the package to the repository without a label.
- Docker manifests and Docker manifest lists have no field for annotations. An OCI index has one.

## Goals / Non-Goals

**Goals:**

- Operators on either architecture install a release or follow `main` without building Adunata.
- What gets published is exactly what CI verified. Publishing builds nothing.
- One version names a release everywhere.

**Non-Goals:**

- An image for Darwin. Docker and Podman on a Mac run `linux/arm64` in a VM.
- Signed images or attestations.
- Pruning old `sha-<commit>` tags.

## Decisions

### 1. Images and release archives for both architectures

Every push to `main` that builds Adunata and passes CI publishes the image as an OCI index for `linux/amd64` and `linux/arm64` under `sha-<commit>` and moves `nightly` to the newest such commit. A push that changes only `openspec/` builds nothing and gets no index. A release publishes the same manifests under its version. `latest` names the release with the highest version among those without a pre-release part. It attaches `adunata-x86_64-linux.tar.gz`, `adunata-aarch64-linux.tar.gz` and `SHA256SUMS`.

Each index carries the OCI annotations `org.opencontainers.image.source`, `revision`, `licenses`, `title` and `description`. The index of a release also carries `version`. It is a new index over the same manifests, so the images stay byte for byte the same and only the digest of the index differs from `sha-<commit>`. The image config gets no labels, because annotations replace them.

An image only for `nightly` would leave releases behind and need a second path later.

Architecturally significant, because operators pull new tags and install on a new architecture. Drafted below as the record "Published artifacts, their tags and the project version".

### 2. One version for the project

`server/Cargo.toml` holds the version. `flake.nix` reads it from there. `client/package.json` carries the same version. A check in the target `lint` fails when `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` disagree or when the version has build metadata, because `+` is not allowed in an image tag. `adunata --version` prints that version, so the binary of a release names its tag. All three files move to `0.0.0` now, so the first release raises them to `0.1.0`.

Leaving `client/package.json` at `0.0.0` would keep a second number that nobody checks.

Architecturally significant, because operators see the version in the tags, the annotations and `adunata --version`. Part of the record "Published artifacts, their tags and the project version".

### 3. Release archive as a derivation

The flake builds `packages.<system>.releaseArchive`. It packs the binary, the example unit, the example environment file, `LICENSE` and the notices with fixed order, owner and time, and writes `adunata-<system>.tar.gz.sha256` next to it. The leg of `build` adds it to its one `nix build` call and uploads it as `release-<system>` instead of the bare tarball, with the same retention. The end-to-end tests unpack it, so they test the published file.

Not architecturally significant, because the content of the archive does not change.

### 4. `publish.yaml` publishes `nightly`

It runs on `workflow_run` after `CI` completed. A job condition lets it continue only if that run succeeded, its event was `push`, its branch was `main` and its head repository is this repository, and only if it uploaded artifacts. A fork cannot push to `main`, so no run of a pull request publishes. The job `index` below has one `concurrency` group per commit, because one shared group would drop pending runs and leave a commit without `sha-<commit>`.

A job `index` downloads `image-<system>` of exactly that run and pushes both images with `skopeo` as `sha-<commit>-amd64` and `sha-<commit>-arm64`, with credentials from a file. It also downloads the `.sha256` file of `release-<system>` from the same run. `docker buildx imagetools create` joins the images into the OCI index `sha-<commit>` with the annotations of decision 1, `org.kammerchor-wernigerode.adunata.ci-run`, which names the run of `CI`, and `org.kammerchor-wernigerode.adunata.archive.<system>.sha256` with the checksum of each archive. The index thereby names the archives that belong to its images, whatever attempt of the run uploaded them. Before it creates the index, it compares the config digest of each pushed image with the artifact and stops on a difference, so no index exists for images that did not arrive intact. If `sha-<commit>` already exists, the job leaves it as it is, so a run again of `CI` never replaces the verified index.

A job `nightly` follows in one `concurrency` group for all runs that never cancels a running job. It walks the first-parent history of `main` from its head and takes the first commit that has an index `sha-<commit>`. It moves `nightly` there if the annotation `revision` of the current `nightly` names another commit. Because each run computes the newest published commit itself, a run that GitHub drops from the queue loses nothing, two runs never move `nightly` at the same time, and a late run never moves it back. A commit that skipped its build or failed its checks never holds `nightly` back. A last step pulls `nightly` without credentials, so a private package fails visibly.

It holds `packages: write`, `actions: read` and `contents: read`, restores no cache, builds nothing and checks out no code. zizmor flags `workflow_run` as a dangerous trigger, because it runs with the rights of this repository after a run that a fork may have started. The job condition excludes those runs, and the workflow neither runs code from the triggering run nor writes its data to `GITHUB_ENV`. The trigger carries `zizmor: ignore[dangerous-triggers]` with this reason.

Publishing from a job in `ci.yaml` would give the workflow that runs for every pull request write access and make the verification of a commit depend on publishing. Moving `nightly` only while the commit is still the head of `main` would let it fall behind after a commit that builds nothing.

Not architecturally significant, because it is CI tooling. Operators see only the tags of decision 1.

### 5. `release.yaml` promotes a tag

It runs on tags `v*`, builds nothing and uses no Nix.

1. The tagged commit is on `main`.
2. The tag equals `v` and the version in `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` of that commit, and that version has a higher SemVer precedence than the one in its first parent, so `1.2.0-rc.1` to `1.2.0` passes and `1.2.0` to `1.1.0` fails. A tag therefore sits only on the commit that raises the version.
3. The index `sha-<commit>` exists, and the run of `CI` that its annotation names still holds the artifacts `release-<system>`.
4. It downloads the archives of both architectures from that run, checks that their checksums equal the annotations of the index, checks each against its `.sha256` file and joins them into `SHA256SUMS`.
5. It creates the GitHub release as a draft with both archives, `SHA256SUMS` and notes from `--generate-notes`. If a draft of an earlier attempt exists, it uploads every one of these files that the draft lacks or holds with another checksum.
6. It creates the index of decision 1 with the annotation `version` and tags it as `<version>`.
7. It publishes the release.
8. A job `latest` in one `concurrency` group for all tag runs, which never cancels a running job, reads the versions of all published stable releases and tags the index of the highest one as `latest`. The first stable release therefore creates `latest`, a newer one moves it, and an older one leaves it and says so. Because each run computes the highest release itself, a run that GitHub drops from the queue loses nothing, and two runs never write `latest` at the same time.

Each step accepts the result of its own earlier attempt, so a failed run can run again. Any failed condition stops the job with a message that names it. When the artifacts expired, the message says that a new commit that raises the version is needed, because a new build would produce other bytes than the published image. It holds `contents: write`, `packages: write` and `actions: read`.

A `workflow_dispatch` with a tag name runs steps 1 to 4 and stops before the first write, so the conditions can be tried on real refs before the first release.

A tag that runs the whole pipeline again would build the release a second time and publish bytes that `main` never verified when the cache is cold. A release by a version bump without a tag would turn every merge of a bump into a release.

Not architecturally significant, because it is CI tooling. Operators see the tags and the version of decisions 1 and 2.

## Draft decision records

### ADR NNNN: Published artifacts, their tags and the project version

**Context.** Ensembles run Adunata on their own hardware, which includes `aarch64-linux` boards. Adunata publishes its image and its release archive only for `x86_64` and only for releases. Nobody can follow `main` without building it. The version lives in several files that nothing compares, and a tag can name any version. ghcr.io creates packages as private, and only a person can make one public.

**Decision.** We will publish every image as one OCI index for `linux/amd64` and `linux/arm64` and every release archive for `x86_64-linux` and `aarch64-linux`. Every push to `main` that builds Adunata and passes all checks publishes `sha-<commit>`, and `nightly` points to the newest of them. A release is a tag `v<version>` on the commit of `main` that raises the version. It publishes the bytes that the checks of that commit verified under its version. `latest` names the highest release without a pre-release part. `server/Cargo.toml` holds the one version of the project. Each index names its source, revision and license in OCI annotations, and a release also its version. The package is public.

This supersedes no record.

**Status.** Accepted

**Consequences.**
- Operators on either architecture install a release or follow `main` without building Adunata or holding credentials.
- `nightly` changes with every merge, including migrations. It suits test instances, not production. Its annotation `revision` names the commit.
- A release names the same version in its tag, its image, its annotations and `adunata --version`. A tag on any other commit, or with another version, publishes nothing.
- A release is possible only while the verified archives of its commit exist. After that, a new commit that raises the version is needed.
- The registry holds `sha-<commit>` per commit and one tag per architecture. They accumulate until someone prunes them.
- A maintainer makes the package public once after its first publish.

## Affected arc42 sections

- `07-deployment-view.md`: the image and the release archive exist for both architectures, the image has the tags `nightly`, `sha-<commit>`, `latest` and the version and carries annotations, and a release is the promotion of a verified commit.
- `08-cross-cutting-concepts.md`: the row `lint` in "Testing" names the version check.
- `09-architecture-decisions.md`: a row for the record.
- `11-risks-and-technical-debt.md`: the NixOS module and the example unit are not tested on `aarch64`, because the hosted arm64 runner has no KVM.

## Risks / Trade-offs

- [The first publish creates a private package, so hosts cannot pull it.] → The last step of `publish.yaml` fails until a maintainer makes it public. The README lists the setting.
- [`publish.yaml` cannot run in a pull request, because `workflow_run` runs only from the default branch.] → A maintainer watches its first run after the merge. The dry run covers `release.yaml`.
- [`sha-<commit>` tags accumulate.] → A cleanup workflow can prune them later.

## Migration Plan

1. Merge after `verify-both-architectures`. The first push to `main` that passes CI publishes `nightly` as a private package.
2. A maintainer sets the package `adunata` to public in its settings on GitHub. This cannot be undone.
3. The first release raises the version to `0.1.0` in `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json`. After its run on `main` passed, a maintainer pushes the tag `v0.1.0` on that commit.

Rollback means reverting the commit. The tags stay in the registry until someone deletes them.
