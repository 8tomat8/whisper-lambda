# Whisper Lambda

This repo hosts the code for the Whisper Lambda, which is a Lambda function that listens on API calls to transcribe audio files.

## Usage

Pull the model dependencies:

```bash
make pull
```

Run the app locally:

```bash
make run

# In another terminal
echo "{ \"model\": \"base\", \"file\": \"$(base64 -i ./sample_1min.mp3)\" }" > req.json
curl -v -X POST \
  'http://127.0.0.1:9001/lambda-url/_/' \
  -H 'content-type: application/json' \
  -d @req.json
```

> NOTE: Before deployment, remove all the extra files from the `model` directory, to avoid huge lambdas.

## Deployment

TBD

## TODO

- [ ] Add deployment instructions
- [ ] Add terrafom scripts
- [ ] Add CDK scripts
- [ ] Add SAM scripts
- [ ] Add CI/CD to this repo
