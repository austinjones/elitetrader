pub const HELP_MESSAGE_BEFORE_OPTS : &'static str = "\
This program collects information about your status in the game,\n\
 and calculates the best possible trade route.";

pub const HELP_MESSAGE_AFTER_OPTS : &'static str = "\
The top N hops are calculated from your current station, and the \n\
 top few trades from those stations are calculated recursively. \n\
 From those possibilities, the best route's first trade is selected, \n\
 and presented to you. The Search Quality flag -q affects the breadth \n\
 of the search process.\n\
\n\
Possible trades are scored by their total profit per minute,\n\
 which the program estimates based on your ship's jump range.\n\
\n\
Here is an example trade hop.  In this example, we start at Chono,\n\
 and buy Imperial Slaves.  We take them to Iben Dock in Peraesii.\n\
 Once the trade is complete, we press <enter> and the credit/min\n\
 analysis is printed.  We were fast!\n\
\n\
-------------------------------------------------------------------\n\
hop 2:	Chono [Siddha Ring]\n\
\n\
buy:	216x Imperial Slaves [Slavery category]\n\
sell:	Peraesii [Iben Dock]\n\
		245.2 Kcr profit for balance 25.49 Mcr\n\
\n\
expect:	20.1 Kcr profit/min over 12.1 mins\n\
		1.1 Kcr profit/ton for 216 tons\n\
		25.8 ly to system, 14 ls to station, 12.1 min total\n\
\n\
wait:	press <enter> once trade is complete.\n\
\n\
actual:	1642246.3 per min over 0.1 minutes\n\
		98.76% faster than expected\n\
-------------------------------------------------------------------\n\
\n\
Give it a try with your ship, and make some credits!\n";