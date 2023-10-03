set dotenv-load

gh_schema:
  curl -f --header "Authorization: bearer $GITHUB_TOKEN" --header "Accept: application/json+v3" https://api.github.com/graphql -o ./schemas/github.graphql --write-out '%{json}'
