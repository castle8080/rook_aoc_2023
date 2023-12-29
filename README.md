# Advent of Code 2023

This is my code for the advent of code 2023.

https://adventofcode.com/2023

* It is mostly in rust.
* I am trying this year to get them done the same day. (see if that goes.)
* I'm not documenting the code really, but trying to keep it clean.

## Dev Notes / Journal
* This has started to help me understand lifetimes better. I have been trying not to copy data uselessly. There are some where I still am though.
* I am trying to reuse and refactor code sometimes after solving.
* Day 5 part 2 was terrible!
* Day 6 gave me a reason to use the quadratic formula.
* Day 8 was very hard to get performance on. I tried for several hours trying to find a sollution which could see when sequences at different start points would align. I heard people solved it with LCM but I couldn't see how that would work in the general case of the problem which had 2 issues:
    1. You could theoretically have more than 1 repeating sequence if the path includes more than 1 node ending in 'Z'.
    2. The repeating sequences and period could have been on offsets which didn't work well.
    * However in the dataset there was only 1 'Z' for each repeating sequence and all of the periods from each sequence matched with the intial offset. This 2nd part is what lets you use LCM. Otherwise you really couldn't. So most of the other fast solutions I saw, didn't work in the general case. I did more research on this and saw that you can solve a system of Diophantine equations. I tried learning more about this and attempted to solve it with a Python library. However, I still would have had to consider different combinations of sequences in the general case.
* Day 10 made me want to give up. It took awhile to get a representation of a graph with squeezing between pipes. First version of code and model was very complex and needed many cases to find connections. Figured out you can map out nodes as locations within spaces. Debugging was an issue. I figured out how to use unicode to draw boxes and show the graphs better, which helped debug.
    * Another interesting thing with day 10 was seeing how to use lifetimes to hold a solution state/search over a borrowed object.
* Day 11 part2 ended up being easy because I realized in part1 I could use a sparse representation of galaxy positions without needing to create a large 2d matrix. It probably helped that I had done work on datascience with python and had used Scipy sparse matrices.
* Day 17 was nice as I got to use a heap from the rust std lib and see how that library works. Kind of annoying to implement Ord for it though.
* Day 21 was despair for part2! It took me a long time to figure out how to do it and I should have realized that I could look at the data and see what I can take advantage of. There were open lines around all the edges and open lines through the center. There was also enough open space for the diagonals. That allowed me to figure out how many boxes would exist for a certain count type.
* Day 23 was also quite hard to get performance. I was able to brute force it, but figured out how to optimize it by simplifying the graph first and condense nodes which don't have choices you need to make.
* Day 24 part 2 was pretty horrible.
    * I spent a huge amount of time trying to find equations to reduce variables.
    * I came up with some early on which should have worked. I was trying to get an equation I could run linear regression on.
    * The answers weren't right so I tried implementing gradient descent.
    * I finally had a break through when I searched for equation solvers and plugged in some equations I had simplified down to on Wolfram Alpha. It gave the right answer. After that I was able to see that my equations and approach to solving them should have worked.
    * I implemented a basic solver and found the main issue was precision and eventually got it working using big decimals.
    * Unfortunately the big decimal library I used was much slower than primitive numbers.
* Day 25 It was nice to complete this one and I implemented Karger's algorithm for finding a graph minimum cut. That was actually fun. I didn't realize how many other data structures you end up using with minimum graph cut, it is really a good exercise in using other daa structures together. It was also interesting to try using a randomized algorithm. It worked well for this day, because you already know what the number cuts you were targetting ahead of time. Because of that you can stop iterations a bit earlier once you find that many cuts.

This was the first AOC I completed all days for. It was very tiring but also very satisfying. I learned a lot more rust because of it, but there are still some things I wish I had tried. I wish I tried multi-threading some algorithms.
