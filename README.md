# Project Purpose
This is a rust port of the [basic action record analysis program prototype](https://github.com/FireChickenProductivity/BasicActionRecordAnalyzer) made to improve performance and because I was curious about rust.

The program takes a talon voice command history as input and outputs the bodies for new commands that might improve productivity by looking for common patterns.

# Usage
The program takes 3 arguments: the path to the history, the maximum command chain size, and the maximum number of recommendations to output. These arguments can also be provided after running the program, which is required if any of the arguments are invalid or not provided.

The maximum command chain size is the number of consecutive commands in the history to consider merging into a single command during analysis. Making this bigger can find longer patterns but takes longer.

If given a maximum number of recommendations of 0, the program will output all recommendations. This usually produces too many unhelpful recommendations. When using a maximum, the program gives you a chance to reject commands you do not like so that it can try to replace them with other good candidates.

The program generates a Recommendations directory outputting each set of recommendations in a text file. It will output some statistics proceeded by a # and the actions for every recommended command.

# Dependencies
The following programs create compatible histories: https://github.com/FireChickenProductivity/BAR and https://github.com/FireChickenProductivity/ArtificialTalonCommandHistoryGenerator. 

Provides actions used by some of the commands it generates: https://github.com/FireChickenProductivity/ActionsForGeneratedCommands.
