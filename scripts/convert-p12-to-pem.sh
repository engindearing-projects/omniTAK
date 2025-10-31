#!/bin/bash
# Convert PKCS#12 (.p12/.pfx) certificates to PEM format for OmniTAK
#
# Usage: ./convert-p12-to-pem.sh <input.p12> [output_dir] [password]
#
# This script extracts:
# - Client certificate (client.pem)
# - Client private key (client.key) in traditional RSA format
# - CA certificate chain (ca.pem)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if input file is provided
if [ -z "$1" ]; then
    echo -e "${RED}Error: No input file specified${NC}"
    echo "Usage: $0 <input.p12> [output_dir] [password]"
    echo ""
    echo "Example:"
    echo "  $0 mycert.p12"
    echo "  $0 mycert.p12 ./certs"
    echo "  $0 mycert.p12 ./certs mypassword"
    exit 1
fi

INPUT_FILE="$1"
OUTPUT_DIR="${2:-.}"
PASSWORD="${3}"

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo -e "${RED}Error: File not found: $INPUT_FILE${NC}"
    exit 1
fi

# Check if openssl is available
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}Error: openssl is not installed${NC}"
    echo "Please install OpenSSL:"
    echo "  - Ubuntu/Debian: sudo apt-get install openssl"
    echo "  - macOS: brew install openssl"
    echo "  - Windows: Download from https://slproweb.com/products/Win32OpenSSL.html"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${GREEN}Converting PKCS#12 to PEM format...${NC}"
echo "Input:  $INPUT_FILE"
echo "Output: $OUTPUT_DIR"
echo ""

# Prepare password argument
PASS_ARGS=""
if [ -n "$PASSWORD" ]; then
    PASS_ARGS="-passin pass:$PASSWORD -passout pass:"
else
    echo -e "${YELLOW}Note: You may be prompted for the .p12 password multiple times${NC}"
    PASS_ARGS="-passout pass:"
fi

# Extract client certificate
echo -e "${GREEN}[1/3]${NC} Extracting client certificate..."
openssl pkcs12 -in "$INPUT_FILE" -out "$OUTPUT_DIR/client.pem" \
    -clcerts -nokeys $PASS_ARGS 2>/dev/null || {
    echo -e "${RED}Failed to extract client certificate${NC}"
    exit 1
}
echo "  ✓ client.pem"

# Extract client private key (in traditional RSA format)
echo -e "${GREEN}[2/3]${NC} Extracting client private key..."
openssl pkcs12 -in "$INPUT_FILE" -out "$OUTPUT_DIR/client_pkcs8.key" \
    -nocerts -nodes $PASS_ARGS 2>/dev/null || {
    echo -e "${RED}Failed to extract private key${NC}"
    exit 1
}

# Convert to traditional RSA format (required by OmniTAK)
if grep -q "BEGIN PRIVATE KEY" "$OUTPUT_DIR/client_pkcs8.key"; then
    echo "  Converting to traditional RSA format..."
    openssl rsa -in "$OUTPUT_DIR/client_pkcs8.key" -out "$OUTPUT_DIR/client.key" \
        -traditional 2>/dev/null || {
        echo -e "${RED}Failed to convert to RSA format${NC}"
        exit 1
    }
    rm "$OUTPUT_DIR/client_pkcs8.key"
else
    # Already in traditional format
    mv "$OUTPUT_DIR/client_pkcs8.key" "$OUTPUT_DIR/client.key"
fi
echo "  ✓ client.key (traditional RSA format)"

# Extract CA certificate chain
echo -e "${GREEN}[3/3]${NC} Extracting CA certificate chain..."
openssl pkcs12 -in "$INPUT_FILE" -out "$OUTPUT_DIR/ca.pem" \
    -cacerts -nokeys $PASS_ARGS 2>/dev/null || {
    echo -e "${YELLOW}Warning: No CA certificates found in .p12 file${NC}"
    touch "$OUTPUT_DIR/ca.pem"
}

# Check if CA file has content
if [ -s "$OUTPUT_DIR/ca.pem" ]; then
    echo "  ✓ ca.pem"
else
    echo -e "${YELLOW}  ! ca.pem is empty (you may need to provide CA certificate separately)${NC}"
fi

echo ""
echo -e "${GREEN}✓ Conversion complete!${NC}"
echo ""
echo "Generated files:"
echo "  - $OUTPUT_DIR/client.pem  (Client certificate)"
echo "  - $OUTPUT_DIR/client.key  (Private key in RSA format)"
echo "  - $OUTPUT_DIR/ca.pem      (CA certificate chain)"
echo ""
echo "You can now use these in your OmniTAK configuration:"
echo ""
echo "  tls:"
echo "    cert_path: \"$OUTPUT_DIR/client.pem\""
echo "    key_path: \"$OUTPUT_DIR/client.key\""
echo "    ca_path: \"$OUTPUT_DIR/ca.pem\""
echo ""

# Verify the key format
if grep -q "BEGIN RSA PRIVATE KEY" "$OUTPUT_DIR/client.key"; then
    echo -e "${GREEN}✓ Key format verified: Traditional RSA format${NC}"
elif grep -q "BEGIN PRIVATE KEY" "$OUTPUT_DIR/client.key"; then
    echo -e "${RED}✗ Warning: Key is in PKCS#8 format, not traditional RSA format${NC}"
    echo "  OmniTAK requires traditional RSA format. Please report this issue."
else
    echo -e "${YELLOW}! Warning: Could not determine key format${NC}"
fi

echo ""
echo -e "${YELLOW}Security reminder:${NC} Keep these files secure and never commit them to version control!"
