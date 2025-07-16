#include <stdio.h>
#include <stdlib.h>

int main() {
    FILE *file = fopen("test_binary.bin", "wb");
    if (file == NULL) {
        perror("Error opening file");
        return 1;
    }
    
    // Write some binary data (not valid UTF-8)
    unsigned char binary_data[] = {
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
        0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8
    };
    
    size_t written = fwrite(binary_data, 1, sizeof(binary_data), file);
    if (written != sizeof(binary_data)) {
        perror("Error writing to file");
        fclose(file);
        return 1;
    }
    
    fclose(file);
    printf("Binary file 'test_binary.bin' created successfully!\n");
    printf("Size: %zu bytes\n", sizeof(binary_data));
    
    return 0;
}