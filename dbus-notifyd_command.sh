#!/bin/bash

# This script is run every time a call comes in. Test the script by running it to hear if the audio is coming from the correct 
# output. The first audio device is set with "plughw:0,0", the second sound card is "plughw:1,0", etc. Use
# only one of the following commands, whichever one works obviously.
#
# aplay works with most distros. Use "aplay -L" to list the devices and find the desired output. My workstation's built in output is this:
#
# front:CARD=SB,DEV=0
#   HDA ATI SB, ALC889 Analog
#   Front speakers
#
# Generally the device that says "front speakers" and is not CARD=Headset is the built in soundcard. The full name is "front:CARD=SB,DEV=0"
#
# plughw:0,0 is an alternate way of setting devices, where the first number is the device number and the second is the output. I do not
# recommend using that method since the devices can be enumerated in a different order.

aplay -q -D front:CARD=SB,DEV=0 ./ring.wav &

# Uncomment and use this on new workstations with fedora if aplay is not available. The device name should be correct for the
# current hardware:
#
# gst-launch-1.0 -q filesrc location=./ring.wav ! wavparse ! audioconvert ! audioresample ! pulsesink device ="alsa_output.pci-0000_0b_00.6.analog-stereo" &
