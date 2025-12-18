#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

printf "${BLUE}[INFO]${NC} Important process launch...\n"

for i in {1..5}
do
    sleep 1
    case $i in
        1) printf "${GREEN}[OK]${NC} Initializing quantum destabilizer...\n" ;;
        2) printf "${GREEN}[OK]${NC} Loading cats into memory...\n" ;;
        3) printf "${YELLOW}[WARN]${NC} Dangerously high caffeine content detected in code...\n" ;;
        4) printf "${YELLOW}[WARN]${NC} System starting to question the meaning of existence...\n" ;;
        5) printf "${RED}[FAIL]${NC} Oops, something went completely wrong!\n" ;;
    esac
done

printf "${RED}Critical error: Process failed (as planned).${NC}\n"

exit 1