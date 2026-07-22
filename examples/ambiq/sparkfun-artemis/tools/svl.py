#!/usr/bin/env python
# SparkFun Variable Loader
# Variable baud rate bootloader for Artemis Apollo3 modules

# Immediately upon reset the Artemis module will search for the timing character
#   to auto-detect the baud rate. If a valid baud rate is found the Artemis will
#   respond with the bootloader version packet
# If the computer receives a well-formatted version number packet at the desired
#   baud rate it will send a command to begin bootloading. The Artemis shall then
#   respond with the a command asking for the next frame.
# The host will then send a frame packet. If the CRC is OK the Artemis will write
#   that to memory and request the next frame. If the CRC fails the Artemis will
#   discard that data and send a request to re-send the previous frame.
# This cycle repeats until the Artemis receives a done command in place of the
#   requested frame data command.
# The initial baud rate determination must occur within some small timeout. Once
#   baud rate detection has completed all additional communication will have a
#   universal timeout value. Once the Artemis has begun requesting data it may no
#   no longer exit the bootloader. If the host detects a timeout at any point it
#   will stop bootloading.

# Notes about PySerial timeout:
# The timeout operates on whole functions - that is to say that a call to
#   ser.read(10) will return after ser.timeout, just as will ser.read(1) (assuming
#   that the necessary bytes were not found)
# If there are no incoming bytes (on the line or in the buffer) then two calls to
#   ser.read(n) will time out after 2*ser.timeout
# Incoming UART data is buffered behind the scenes, probably by the OS.

# ***********************************************************************************
#
# Imports
#
# ***********************************************************************************

import argparse
import serial
import serial.tools.list_ports as list_ports
import sys
import time
import math
import os.path
from sys import exit

SCRIPT_VERSION_MAJOR = "1"
SCRIPT_VERSION_MINOR = "7"

# ***********************************************************************************
#
# Commands
#
# ***********************************************************************************
SVL_CMD_VER = 0x01  # version
SVL_CMD_BL = 0x02  # enter bootload mode
SVL_CMD_NEXT = 0x03  # request next chunk
SVL_CMD_FRAME = 0x04  # indicate app data frame
SVL_CMD_RETRY = 0x05  # request re-send frame
SVL_CMD_DONE = 0x06  # finished - all data sent

barWidthInCharacters = 50  # Width of progress bar, ie [###### % complete

crcTable = (
    0x0000, 0x8005, 0x800F, 0x000A, 0x801B, 0x001E, 0x0014, 0x8011,
    0x8033, 0x0036, 0x003C, 0x8039, 0x0028, 0x802D, 0x8027, 0x0022,
    0x8063, 0x0066, 0x006C, 0x8069, 0x0078, 0x807D, 0x8077, 0x0072,
    0x0050, 0x8055, 0x805F, 0x005A, 0x804B, 0x004E, 0x0044, 0x8041,
    0x80C3, 0x00C6, 0x00CC, 0x80C9, 0x00D8, 0x80DD, 0x80D7, 0x00D2,
    0x00F0, 0x80F5, 0x80FF, 0x00FA, 0x80EB, 0x00EE, 0x00E4, 0x80E1,
    0x00A0, 0x80A5, 0x80AF, 0x00AA, 0x80BB, 0x00BE, 0x00B4, 0x80B1,
    0x8093, 0x0096, 0x009C, 0x8099, 0x0088, 0x808D, 0x8087, 0x0082,
    0x8183, 0x0186, 0x018C, 0x8189, 0x0198, 0x819D, 0x8197, 0x0192,
    0x01B0, 0x81B5, 0x81BF, 0x01BA, 0x81AB, 0x01AE, 0x01A4, 0x81A1,
    0x01E0, 0x81E5, 0x81EF, 0x01EA, 0x81FB, 0x01FE, 0x01F4, 0x81F1,
    0x81D3, 0x01D6, 0x01DC, 0x81D9, 0x01C8, 0x81CD, 0x81C7, 0x01C2,
    0x0140, 0x8145, 0x814F, 0x014A, 0x815B, 0x015E, 0x0154, 0x8151,
    0x8173, 0x0176, 0x017C, 0x8179, 0x0168, 0x816D, 0x8167, 0x0162,
    0x8123, 0x0126, 0x012C, 0x8129, 0x0138, 0x813D, 0x8137, 0x0132,
    0x0110, 0x8115, 0x811F, 0x011A, 0x810B, 0x010E, 0x0104, 0x8101,
    0x8303, 0x0306, 0x030C, 0x8309, 0x0318, 0x831D, 0x8317, 0x0312,
    0x0330, 0x8335, 0x833F, 0x033A, 0x832B, 0x032E, 0x0324, 0x8321,
    0x0360, 0x8365, 0x836F, 0x036A, 0x837B, 0x037E, 0x0374, 0x8371,
    0x8353, 0x0356, 0x035C, 0x8359, 0x0348, 0x834D, 0x8347, 0x0342,
    0x03C0, 0x83C5, 0x83CF, 0x03CA, 0x83DB, 0x03DE, 0x03D4, 0x83D1,
    0x83F3, 0x03F6, 0x03FC, 0x83F9, 0x03E8, 0x83ED, 0x83E7, 0x03E2,
    0x83A3, 0x03A6, 0x03AC, 0x83A9, 0x03B8, 0x83BD, 0x83B7, 0x03B2,
    0x0390, 0x8395, 0x839F, 0x039A, 0x838B, 0x038E, 0x0384, 0x8381,
    0x0280, 0x8285, 0x828F, 0x028A, 0x829B, 0x029E, 0x0294, 0x8291,
    0x82B3, 0x02B6, 0x02BC, 0x82B9, 0x02A8, 0x82AD, 0x82A7, 0x02A2,
    0x82E3, 0x02E6, 0x02EC, 0x82E9, 0x02F8, 0x82FD, 0x82F7, 0x02F2,
    0x02D0, 0x82D5, 0x82DF, 0x02DA, 0x82CB, 0x02CE, 0x02C4, 0x82C1,
    0x8243, 0x0246, 0x024C, 0x8249, 0x0258, 0x825D, 0x8257, 0x0252,
    0x0270, 0x8275, 0x827F, 0x027A, 0x826B, 0x026E, 0x0264, 0x8261,
    0x0220, 0x8225, 0x822F, 0x022A, 0x823B, 0x023E, 0x0234, 0x8231,
    0x8213, 0x0216, 0x021C, 0x8219, 0x0208, 0x820D, 0x8207, 0x0202)
# ***********************************************************************************
#
# Compute CRC on a byte array
#
# ***********************************************************************************


def get_crc16(data):

    # Table and code ported from Artemis SVL bootloader
    crc = 0x0000
    data = bytearray(data)
    for ch in data:
        tableAddr = ch ^ (crc >> 8)
        CRCH = (crcTable[tableAddr] >> 8) ^ (crc & 0xFF)
        CRCL = crcTable[tableAddr] & 0x00FF
        crc = CRCH << 8 | CRCL
    return crc


# ***********************************************************************************
#
# Wait for a packet
#
# ***********************************************************************************
def wait_for_packet(ser):

    packet = {'len': 0, 'cmd': 0, 'data': 0, 'crc': 1, 'timeout': 1}

    n = ser.read(2)  # get the number of bytes
    if(len(n) < 2):
        return packet

    packet['len'] = int.from_bytes(n, byteorder='big', signed=False)    #
    payload = ser.read(packet['len'])

    if(len(payload) != packet['len']):
        return packet

    # all bytes received, so timeout is not true
    packet['timeout'] = 0
    # cmd is the first byte of the payload
    packet['cmd'] = payload[0]
    # the data is the part of the payload that is not cmd or crc
    packet['data'] = payload[1:packet['len']-2]
    # performing the crc on the whole payload should return 0
    packet['crc'] = get_crc16(payload)

    return packet

# ***********************************************************************************
#
# Send a packet
#
# ***********************************************************************************


def send_packet(ser, cmd, data):
    data = bytearray(data)
    num_bytes = 3 + len(data)
    payload = bytearray(cmd.to_bytes(1, 'big'))
    payload.extend(data)
    crc = get_crc16(payload)
    payload.extend(bytearray(crc.to_bytes(2, 'big')))

    ser.write(num_bytes.to_bytes(2, 'big'))
    ser.write(bytes(payload))


# ***********************************************************************************
#
# Setup: signal baud rate, get version, and command BL enter
#
# ***********************************************************************************
def phase_setup(ser):

    baud_detect_byte = b'U'

    verboseprint('\nPhase:\tSetup')

    # Handle the serial startup blip
    ser.reset_input_buffer()
    verboseprint('\tCleared startup blip')

    ser.write(baud_detect_byte)             # send the baud detection character

    packet = wait_for_packet(ser)
    if(packet['timeout'] or packet['crc']):
        return False  # failed to enter bootloader

    twopartprint('\t', 'Got SVL Bootloader Version: ' +
                 str(int.from_bytes(packet['data'], 'big')))
    verboseprint('\tSending \'enter bootloader\' command')

    send_packet(ser, SVL_CMD_BL, b'')

    return True

    # Now enter the bootload phase


# ***********************************************************************************
#
# Bootloader phase (Artemis is locked in)
#
# ***********************************************************************************
def phase_bootload(ser):

    startTime = time.time()
    frame_size = 512*4

    resend_max = 4
    resend_count = 0

    verboseprint('\nPhase:\tBootload')

    with open(args.binfile, mode='rb') as binfile:
        application = binfile.read()
        total_len = len(application)

        total_frames = math.ceil(total_len/frame_size)
        curr_frame = 0
        progressChars = 0

        if (not args.verbose):
            print("[", end='')

        verboseprint('\thave ' + str(total_len) +
                     ' bytes to send in ' + str(total_frames) + ' frames')

        bl_done = False
        bl_succeeded = True
        while((bl_done == False) and (bl_succeeded == True)):

            # wait for indication by Artemis
            packet = wait_for_packet(ser)
            if(packet['timeout'] or packet['crc']):
                verboseprint('\n\tError receiving packet')
                verboseprint(packet)
                verboseprint('\n')
                bl_succeeded = False
                bl_done = True

            if(packet['cmd'] == SVL_CMD_NEXT):
                # verboseprint('\tgot frame request')
                curr_frame += 1
                resend_count = 0
            elif(packet['cmd'] == SVL_CMD_RETRY):
                verboseprint('\t\tRetrying...')
                resend_count += 1
                if(resend_count >= resend_max):
                    bl_succeeded = False
                    bl_done = True
            else:
                print('Timeout or unknown error')
                bl_succeeded = False
                bl_done = True

            if(curr_frame <= total_frames):
                frame_data = application[(
                    (curr_frame-1)*frame_size):((curr_frame-1+1)*frame_size)]
                if(args.verbose):
                    verboseprint('\tSending frame #'+str(curr_frame) +
                                 ', length: '+str(len(frame_data)))
                else:
                    percentComplete = curr_frame * 100 / total_frames
                    percentCompleteInChars = math.ceil(
                        percentComplete / 100 * barWidthInCharacters)
                    while(progressChars < percentCompleteInChars):
                        progressChars = progressChars + 1
                        print('#', end='', flush=True)
                    if (percentComplete == 100):
                        print("]", end='')

                send_packet(ser, SVL_CMD_FRAME, frame_data)

            else:
                send_packet(ser, SVL_CMD_DONE, b'')
                bl_done = True

        if(bl_succeeded == True):
            twopartprint('\n\t', 'Upload complete')
            endTime = time.time()
            bps = total_len / (endTime - startTime)
            verboseprint('\n\tNominal bootload bps: ' + str(round(bps, 2)))
        else:
            twopartprint('\n\t', 'Upload failed')

        return bl_succeeded


# ***********************************************************************************
#
# Help if serial port could not be opened
#
# ***********************************************************************************
def phase_serial_port_help():
    devices = list_ports.comports()

    # First check to see if user has the given port open
    for dev in devices:
        if(dev.device.upper() == args.port.upper()):
            print(dev.device + " is currently open. Please close any other terminal programs that may be using " +
                  dev.device + " and try again.")
            exit()

    # otherwise, give user a list of possible com ports
    print(args.port.upper() +
          " not found but we detected the following serial ports:")
    for dev in devices:
        if 'CH340' in dev.description:
            print(
                dev.description + ": Likely an Arduino or derivative. Try " + dev.device + ".")
        elif 'FTDI' in dev.description:
            print(
                dev.description + ": Likely an Arduino or derivative. Try " + dev.device + ".")
        elif 'USB Serial Device' in dev.description:
            print(
                dev.description + ": Possibly an Arduino or derivative.")
        else:
            print(dev.description)


# ***********************************************************************************
#
# Main function
#
# ***********************************************************************************
def main():
    try:
        num_tries = 3

        print('\n\nArtemis SVL Bootloader')

        verboseprint("Script version " + SCRIPT_VERSION_MAJOR +
                     "." + SCRIPT_VERSION_MINOR)

        if not os.path.exists(args.binfile):
            print("Bin file {} does not exist.".format(args.binfile))
            exit()

        bl_success = False
        entered_bootloader = False

        for _ in range(num_tries):

            with serial.Serial(args.port, args.baud, timeout=args.timeout) as ser:

                # startup time for Artemis bootloader   (experimentally determined - 0.095 sec min delay)
                t_su = 0.15

                time.sleep(t_su)        # Allow Artemis to come out of reset

                # Perform baud rate negotiation
                entered_bootloader = phase_setup(ser)

                if(entered_bootloader == True):
                    bl_success = phase_bootload(ser)
                    if(bl_success == True):     # Bootload
                        #print("Bootload complete!")
                        break
                else:
                    verboseprint("Failed to enter bootload phase")

            if(bl_success == True):
                break

        if(entered_bootloader == False):
            print(
                "Target failed to enter bootload mode. Verify the right COM port is selected and that your board has the SVL bootloader.")

    except serial.SerialException:
        phase_serial_port_help()

    exit()


# ******************************************************************************
#
# Main program flow
#
# ******************************************************************************
if __name__ == '__main__':

    parser = argparse.ArgumentParser(
        description='SparkFun Serial Bootloader for Artemis')

    parser.add_argument('port', help='Serial COMx Port')

    parser.add_argument('-b', dest='baud', default=115200, type=int,
                        help='Baud Rate (default is 115200)')

    parser.add_argument('-f', dest='binfile', default='',
                        help='Binary file to program into the target device')

    parser.add_argument("-v", "--verbose", default=0, help="Enable verbose output",
                        action="store_true")

    parser.add_argument("-t", "--timeout", default=0.50, help="Communication timeout in seconds (default 0.5)",
                        type=float)

    if len(sys.argv) < 2:
        print("No port selected. Detected Serial Ports:")
        devices = list_ports.comports()
        for dev in devices:
            print(dev.description)

    args = parser.parse_args()

    # Create print function for verbose output if caller deems it: https://stackoverflow.com/questions/5980042/how-to-implement-the-verbose-or-v-option-into-a-script
    if args.verbose:
        def verboseprint(*args):
            # Print each argument separately so caller doesn't need to
            # stuff everything to be printed into a single string
            for arg in args:
                print(arg, end='', flush=True),
            print()
    else:
        verboseprint = lambda *a: None      # do-nothing function

    def twopartprint(verbosestr, printstr):
        if args.verbose:
            print(verbosestr, end='')

        print(printstr)

    main()
