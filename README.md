# Combine Racers

For Bevy Jam #2. [v0.1.11](https://github.com/rparrett/combine-racers/tree/v0.1.11) is the version that was submitted.

[Play on itch.io](https://euclidean-whale.itch.io/combine-racers)

Do tricks to go fast in your Combine Harvester! Go fast longer by doing a different trick than the last one.

If you're building from source, the leaderboard will be unavailable.

## Acknowledgements

`7th-race-aiteru-sawato.ogg` is an original composition by [Aiteru Sawato](https://www.youtube.com/channel/UCXkaOsXAVvxY2HFFRt7PjPQ) produced for this project and redistributed here with their explicit permission.

`NanumPenScript-Tweaked.ttf` is derived from [Nanum Pen Script](https://fonts.adobe.com/fonts/nanum-pen-script) and is licensed under the SIL Open Font License.

All other assets are original creations by me for this project.

The [leaderboard server](https://jornet.vleue.com/) was kindly provided by [mockersf](https://github.com/sponsors/mockersf).

## TODO

- [X] (Pre-release) Reset leaderboard
- [ ] (Stretch goal) Boost gauge
- [ ] (Stretch goal) Textures for track and finish line
- [ ] (Stretch goal) Sticky patches on track
- [ ] (Stretch goal) Barrel roll trick
- [ ] (Stretch goal) Lava at bottom of map
- [ ] (Stretch goal) Parallax background or skybox
- [X] (Stretch goal) Speedometer
- [X] (Stretch goal) Navigate UI with gamepad
- [X] (Stretch goal) Navigate UI with keyboard
- [X] (Stretch goal) Use our own name generator for the leaderboard
- [ ] (Probably not happening) Add corn or something

## Track workflow

Asset sources are in the [combine-racers-assets](https://github.com/rparrett/combine-racers-assets) repo.

- Create path on grid in inkscape
- Track segments should be separate paths
- Import svg into `combine-racers-geometry-nodes.blend`.
- Scale by 500 (s500) and (g) move start of track just below origin
- Select all and apply all transformations (command-a)
- Apply geometry node modifier to track segments
- Rename track segment curves to `Track`
- Add a cube and name the mesh `FinishLineCollider`
- Export GLTF. Check remember. Uncheck +Y Up. Check "apply modifiers." Uncheck animations, etc.
