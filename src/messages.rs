pub const HELP_MESSAGE_BEFORE_OPTS : &'static str = "\
This program collects information about your status in the game,\n\
and calculates the best possible trade route.\n\
\n\
It is useful to set the -c Credit, -r Jump Range, -p Ship Size, and \n\
-m Min Balance args on the command line, as they don't change often.\n\
The tool will interactively prompt you for any remaining information,\n\
such as the Current Station and Credit Balance.\n\
\n\
Set Minimum Balance argument carefully.  You should allow your rebuy\n\
cost plus a full load of expensive cargo, or two of each to be safe.\n\
If you set the balance too low, you might end up broke in a Sidewinder!";

pub const HELP_MESSAGE_AFTER_OPTS : &'static str = "\
The top few trades are calculated from your current station, and the\n\
top few trades from those stations are calculated, and so on.\n\
From all the possible trade routes, the first trade from 
the best route is presented to you.\n\
\n\
The Search Quality flag -q affects the depth of the search process.\n\
\n\
Possible trades are scored by their total profit per minute,\n\
 which the program estimates based on your ship's jump range.\n\
\n\
Here is an example trade hop.  In this example, we start at Giger Hub,\n\
and buy Palladium.  We take it to Iben Dock in Peraesii.\n\
Once the trade is complete, we press <enter> and the credit/min\n\
analysis is printed.\n\
\n\
-------------------------------------------------------------------\n\
Starting route search from Peraesii [Giger Hub] ...\n\
hop 1:	Peraesii [Giger Hub]\n\
\n\
buy:	216x Palladium [Metals]\n\
sell:	Chono [Siddha Ring]\n\
248.9 Kcr profit for balance 15.25 Mcr\n\
\n\
expect:	28.0 Kcr profit/min over 9.1 mins\n\
1.2 Kcr profit/ton for 216 tons\n\
25.8 ly to system, 9 ls to station\n\
\n\
wait:	press <enter> once trade is complete.\n\
\n\
actual:	29.1 Kcr per min over 8.6 minutes\n\
5.82% faster than expected\n\
-------------------------------------------------------------------
\n\
We made 29 thousand credits per minute over 8.6 minutes.\n\
Give it a try with your ship!\n";
