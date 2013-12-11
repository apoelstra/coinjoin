
#include <stdlib.h>
#include <openssl/sha.h>

unsigned char *csha256_sum (unsigned char *input, size_t len)
{
  unsigned char *rv = malloc (65);

  if (rv != NULL) {
    SHA256_CTX ctx;
    SHA256_Init (&ctx);
    SHA256_Update (&ctx, input, len);
    SHA256_Final (rv, &ctx);
  }

  return rv;
}

void csha256_destroy (unsigned char *hash)
{
  free (hash);
}

