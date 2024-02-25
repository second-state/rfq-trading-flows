# rfq-trading-flows

rfq-trading-flows is a trading bot running on [flows.network](https://flows.network/) and using [rfq-markeyplace](https://github.com/second-state/rfq-marketplace) to exchange tokens. The bot follows a simple "buy low and sell high" strategy.

## Deploy the trading bot

To run the bot, we will use [flows.network](https://flows.network/), a serverless platform that makes deploying your app quick and easy in just three steps.

## Prerequisite

You will need a wallet private key. If you do not already have one, use [Metamask](https://metamask.io/) to create it.

#### Fork this repo and write your own code

Fork [this repo](https://github.com/second-state/rfq-trading-flows). 

#### Deploy the code on flows.network

1. Sign up for an account for deploying flows on [flows.network](https://flows.network/). It's free.
2. Click on the "Create a Flow" button to start deploying the web service.
3. Authenticate the [flows.network](https://flows.network/) to access the `rfq-trading-flows` repo you just forked. 
4. Click on the Advanced text and you will see more settings including branch and environment variables. In this example, we have four variables to fill in, `CONTRACT_ADDRESS` is rfq-marketplace contract address, and `PRIVATE_KEY` is the bot's wallet private key.
The default network is cybermiles. If you want to change the network, you can set `RPC_NODE_URL` and `CHAIN_ID` variables.
<img width="964" alt="image" src="https://i.imgur.com/nbsTeXf.png">

5. Click the Deploy button to deploy your function.

### Configure SaaS integrations

After that, the flows.network will direct you to configure the SaaS integration required by your flow. Here we can see that there is no SaaS needs to be connected since it's a lambda service. Just click the Check button to see your flows details.

<img width="964" alt="image" src="https://user-images.githubusercontent.com/45785633/226959151-0e8a159a-02b3-4130-b7b5-8831b65c8d75.png">

## Try this demo

This repository includes two trading bots. The first one is following the "buy low and sell high" strategy. The second is a random response bot. In each flows function, You can select one of the bots to run. Or you can create two flows functions to run the two bots simultaneously. It will show the first bot can make money from the second bot.

### Buy low and sell high bot

Following the above operations, you can deploy the bot to [flows.network](https://flows.network/), and then you need to set some parameters to control your bot.
Finally, you can through regular call the endpoint to run the bot.

#### Setting parameter

After deploying your function, you can click the setting button to set more environments for your function. This bot provided the following parameter let you can set. </br>

1. `BASE`: The address of the base currency of the currency pair.
2. `QUOTE`: The address of the quote currency of the currency pair.
3. `PROFIT_SPREAD`: Profit amount in each trading round.
4. `EXCHANGE_QUANTITY`: Exchange in base currency quantity in each trading round.
5. `QUANTITY`: Exchange out quote currency quantity in each trading round.

<img width="964" alt="image" src="https://i.imgur.com/2jMGxGa.png">

#### Setting webhook

We need an app to regularly call our Webhook Endpoint to trigger this bot. In the example, we will use [easycron](https://www.easycron.com/) to do that. You can change it to any other app that has the same function.
On the [easycron](https://www.easycron.com/) website, click the "Create Cron Job" button. Copy and paste the endpoint URL to the "URL to call" and add `/trigger`. Then click the "HTTP" button setting the HTTP method to POST. Click the "Create Cron Job" to create a job. </br>

You can set another parameter for your needs. It recommends executing frequency larger than 1 minute and timeout larger than 1 minute.

<img width="964" alt="image" src="https://i.imgur.com/Z8v3CWI.png">
<img width="964" alt="image" src="https://i.imgur.com/8rLUMiU.png">

#### The strategy of this bot

1. The bot creates an exchange request that exchanges out `QUANTITY` quantity of quote currency.
2. Check if there is a response quantity of base currency larger than `EXCHANGE_QUANTITY`.
3. Accept the response and withdraw.
4. Creating a reverse exchange request that exchanges out the number of withdrawals of the base currency.
5. Check if there is a response quantity of quote currency larger than `QUANTITY` + `PROFIT_SPREAD`.
6. Accept the response and withdraw.
7. Repeat the above steps.

In each trading round, we can use `QUANTITY` quantity of quote currency to profit `PROFIT_SPREAD` quantity of quote currency.

### Random response bot

#### Setting parameter

After deploying your function, you can click the setting button to set more environments for your function. This bot provided the following parameter let you can set. </br>

1. `COOLING_TIME`: Rest time after each round of trading, the unit is second.
2. `BASE`: The address of the base currency of the currency pair.
3. `QUOTE`: The address of the quote currency of the currency pair.
4. `MIN_QUOTE_QUANTITY`: The minimum number of quote currency exchanges out.
5. `MAX_QUOTE_QUANTITY`: The maximum number of quote currency exchanges out.
6. `MIN_BASE_QUANTITY`: The minimum number of base currency exchanges out.
7. `MAX_BASE_QUANTITY`: The maximum number of base currency exchanges out.

<img width="964" alt="image" src="https://i.imgur.com/pqyleFz.png">

#### Setting webhook

The same method as the [above setting](#setting-webhook). But "URL to call" needs to change to endpoint URL add `/random-response`. </br>

You also can set another parameter for your needs. It recommends executing frequency larger than `COOLING_TIME` and timeout larger than 1 minute.

#### The strategy of this bot

This bot will query the latest request. If the request `token_in` is the base currency, the bot will submit a response that quantity is a random number between `MIN_BASE_QUANTITY` and `MAX_BASE_QUANTITY`. If the request `token_in` is the quote currency, the bot will submit a response that quantity is a random number between `MIN_QUOTE_QUANTITY` and `MAX_QUOTE_QUANTITY`.</br>
After `COOLING_TIME` seconds, no matter if the response is accepted, the bot will withdraw. Then repeat the above operations.

### Reset status

Since the bot will store the status in each round. If something unexpected happens, let the robot make unexpected operations. You can use curl to call the reset API to reset the bot status. The reset API path is `reset-state`.
``` shell
curl "https://code.flows.network/webhook/your-url/reset-state"
```
### Demo result

If you run the two bots simultaneously and set the parameter following the above image. You can see that the "buy low and sell high" bot always creates new requests. Once the "random response" submits the larger quantity response, the "buy low and sell high" bot will accept it and create a new request.

<img width="964" alt="image" src="https://i.imgur.com/HkHP4eW.png">

The detailed flows function log in [here](log.txt)

> [flows.network](https://flows.network/) is still in its early stages. We would love to hear your feedback!

## Others


To build locally, make sure you have installed Rust and added `wasm32-wasi` target.

```
cd flows
cargo build target wasm32-wasi --release
```
