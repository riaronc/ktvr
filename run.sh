#!/bin/bash
# run.sh

set -e

# Default values
ENV="dev"
BUILD=false
INIT=false

# Function to display usage
usage() {
    echo "Usage: $0 -e [dev|stage|prod] [ -b ] [ --init ]"
    echo "  -e    Set environment (dev, stage, prod). Default is dev."
    echo "  -b    Build the Docker images before running."
    echo "  --init Initialize the environment (create network and services)."
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]
do
  key="$1"

  case $key in
    -e )
      ENV="$2"
      shift # past argument
      shift # past value
      ;;
    -b )
      BUILD=true
      shift # past argument
      ;;
    --init )
      INIT=true
      shift # past argument
      ;;
    * )
      echo "Unknown option: $1"
      usage
      ;;
  esac
done

# Validate ENV
if [[ ! "$ENV" =~ ^(dev|stage|prod)$ ]]; then
    echo "Invalid environment: $ENV" 1>&2
    usage
fi

# Determine env file
ENV_FILE=".env.${ENV}"

if [ ! -f "$ENV_FILE" ]; then
    echo "Environment file $ENV_FILE does not exist." 1>&2
    exit 1
fi

# Set network name based on environment
NETWORK_NAME="ktvr_app_network_${ENV}"

if [ "$INIT" = true ]; then
    COMPOSE_FILE="docker-compose-init.yml"
else
    COMPOSE_FILE="docker-compose.yml"
fi

# If build is requested or init is being run
if [ "$BUILD" = true ]; then
    docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up --build -d
else
    docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d
fi
