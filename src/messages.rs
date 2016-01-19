pub const HELP_MESSAGE_BEFORE_OPTS : &'static str = "\
This program collects information about your status in the game,\n\
and calculates the best possible trade route.\n\
\n\
It is useful to set the -c Credit, -r Jump Range, -p Ship Size, and \n\
-m Min Balance args on the command line, as they don't change often.\n\
The tool will interactively prompt you for any remaining information,\n\
such as the Current Station and Credit Balance. \n\
\n\
Alternatively, you can provide the --edce parameter with the path to an \n\
installed copy of Elite Dangerous Companion Emulator. With EDCE integration, 
EliteTrader can automatically load your player state at startup, and\n\
commodity prices during a trade.  It is useful to set the --autoaccept\n\
parameter when EDCE is configured, as you never need to fix a buy price.\n\
Note: you need python 3, and the Requests library (pip install requests)\n\
installed, and must configure the client outside of Elite Trader.  \n\
See: https://github.com/Andargor/edce-client for setup instructions.\n\
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
Enumerating 7 trades per station to a depth of 9 hops ...\n\
Total routes to examine: 40.35 M\n\
-------------------------------------------------------------------\n\
wait:   calculating ...\n\
-------------------------------------------------------------------
hop 0:  11:15AM, estimated 45.6 Kcr profit/min over next 72 minutes\n\
\n\
buy:    Peraesii [Giger Hub]\n\
        Metals [Palladium] at 13.8 Kcr x 216\n\
supply: 24.1 Ktn [48.40 Mcr over 18.12 hours]\n\
\n\
sell:   Maiki [Zenbei Port] at 15.8 Kcr\n\
        434.6 Kcr profit for balance 150.43 Mcr\n\
\n\
expect: 44.5 Kcr profit/min from trade over 9.8 mins\n\
        2.0 Kcr profit/ton for 216 tons\n\
        61 ly to system [5.2 mins]\n\
        925 ls to station [4.5 mins]\n\
\n\
start:  enter) to accept trade\n\
        u) to update buy price (13762)\n\
        n) for new trade\n\
        q) to quit\n\
\n\
end:    enter) to complete trade\n\
        u) to update sell price (15774)\n\
        q) to complete route\n\
\n\
actual: 142.0% of expected - 63.2 Kcr profit/min from trade\n\
        100.0% of expected - 2.0 Kcr profit per ton\n\
        70.4% of expected - 6.87 minutes\n\
-------------------------------------------------------------------\n\
\n\
We made 63 thousand credits per minute over 7 minutes.\n\
Give it a try with your ship!\n";
