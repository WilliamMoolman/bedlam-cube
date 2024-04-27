# Bedlam Cube Solving
A Bedlam Cube is a puzzle where the goals is to assemble various shapes built of cubes into a 4x4x4 cube. According to the manufacturer there exist 19,186 unique solutions (accounting for reflections). The goal of this repository is to recreate that number, and experiment with various game solving techniques.

![](cube.jpg)

## Introduction to Previous Work
This repository was created alongside [bungogood](https://github.com/bungogood/puzzle-cubes), who initially worked on solving the Bedlam cube. The work in this repository was built by myself, but the methodologies and techniques used were joinly developed and discuessed.

Much of the inspiration comes from [Scott Kurowski](http://www.scottkurowski.com/BedlamCube/) who is one of the earliest people to work on solving the cube programmatically. The encoding scheme for the pieces (as seen in `pieces.csv`) was taken from him.

## Methodology
From the initial set of 13 pieces we can find a unique set of possible rotations (up to 24 -- see rotations of a cube) for each piece, and for each of those rotations we can find the possible placements of each piece in the overall 4x4x4 cube. 

A naive tree search is not feasible, as many pieces have up to 423 possible placements across their rotations, giving us a worse-case branching factor of over 400. We therefore need to narrow this search space. An initial implementation tried to use entropy (See https://wikipedia.com/Entropy_(Information_Theory)) to direct our tree search, and greedily pick the next move (placement) to maximise entropy (analogous to picking the Most Constrained Variable). At a search depth of 1 and 2 the puzzle failed to solve, and a depth of 3 was not computationally viable.

To better reduce the search space, we opted to use a common heuristic in human puzzle solving -- starting from the corners. By permuting through all pieces that fit in the 8 cube corners our program could start discovering possible solutions, with an average rate of 0.42 seconds/solution. We find the expected 460464 total solutions, from which we know there are 19186 rotationally unique solutions (division by 24 rotations of a cube).

## Results
These results are timed from a single run. Machine that ran the program was a  Intel i5-7600 @ 4.100GHz with 16Gb of DDR4 memory.

| Version | Time (s) | Solutions | Rotationally Unique | Notes |
| ------- | -------- | --------- | ------------------- | ----- |
| Current | 195017   | 460464    | 19186 (derived)     | ----- |

