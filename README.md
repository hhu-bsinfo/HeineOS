# HeineOS

## Introduction
HeineOS is a teaching operating system for the x86_64 architecture written in Rust.
It serves as the basis for the operating system courses at Heinrich Heine University Düsseldorf, where students build their own operating system from scratch.

This is a project by the [Operating Systems group](https://www.cs.hhu.de/en/research-groups/operating-systems.html) at the *Heinrich Heine University Düsseldorf*.

<p align="center">
  <a href="https://www.uni-duesseldorf.de/home/en/home.html"><img src="media/hhu.svg" width=300></a>
</p>

## Lessons
HeineOS is built gradually over the course of 14 lessons, each introducing new concepts and features.
The lessons are available at the branches of this repository.  
The first seven lessons are part of the first course *Operating Systems Development* (Betriebssystementwicklung),
while the remaining seven lessons are part of the second course *Isolation and Protection in Operating Systems* (Isolation und Schutz in Betriebssystemen).

- Lesson 1: [Input/Output](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-1)
- Lesson 2: [Memory Management & PC Speaker](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-2)
- Lesson 3: [Interrupts](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-3)
- Lesson 4: [Cooperative Multitasking](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-4)
- Lesson 5: [Preemptive Multitasking](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-5)
- Lesson 6: [Filesystem & Porting a Game Boy Emulator](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-6)
- Lesson 7: [Implement your own application](https://github.com/hhu-bsinfo/HeineOS/tree/lesson-7)

## Hall of Fame
Each year, exceptional submissions are included in our *Hall of Fame*.

### 2016
This was the first year for the course *Operating Systems Development*.
At that time, the system was called *hhuOS* and developed in C++.
It targeted the 32-bit x86-architecture and used CGA text mode for screen output, but students were given the option to switch to a VESA framebuffer by the end of the course.
In 2017 *hhuOS* was spun off into its own project still developed in its own [repository](https://github.com/hhuOS/hhuOS) and the operating system for this course was renamed to *hhuTOS* (TOS stands for **T**eaching **O**perating **S**ystem).

- Jochen Peters - Fractals
<video src="https://github.com/user-attachments/assets/cd595508-7768-4b8d-aa00-9e89fb238cf4"></video>

- Filip Krakowski - Space Invaders
<video src="https://github.com/user-attachments/assets/cfb8f07b-7211-4dc0-a3a9-8e38b979a828"></video>

### 2023
By 2023, students could choose whether to implement their operating system in C++ (hhuTOSc) or Rust (hhuTOSr).
The operating system switched to the 64-bit x86_64-architecture but still targeted older computers with a classic BIOS (no UEFI).
It still used CGA text mode for screen output, with the optional VESA framebuffer extension.

- Jannik Esser - CGA Space Invaders
<video src="https://github.com/user-attachments/assets/06bce17d-6a8f-4096-b348-8c3d26ed2bdd"></video> 

- Tim Laurischkat - PAC-MAN
<video src="https://github.com/user-attachments/assets/ee07a415-1bd4-424b-ac96-631fbfced11a"></video>

### 2024
In 2024, C++ was completely replaced by Rust.

- Christoph Jung - Sound Blaster
<video src="https://github.com/user-attachments/assets/cd8cd3ce-6951-4c9e-b866-bde314d92846"></video>

- Lukas Rose - Text Adventure
<video src="https://github.com/user-attachments/assets/38a5d569-c471-41a8-b9ff-1f84b57e7b25"></video>

- Richard Schweitzer - Tetris
<video src="https://github.com/user-attachments/assets/f259825d-37db-46b4-a432-b9c19cb9538b"></video>

### 2025
Erik Gal - PAC-MAN
<video src="https://github.com/user-attachments/assets/73480e12-d03a-4bab-a24c-600349ac0275"></video>

Lukas Lang - Pokémon Reimplementation (No Emulator!)
<video src="https://github.com/user-attachments/assets/6ac2c105-1f27-493f-9448-1da3c8f2ec97"></video>

Stephan Schmidt - Minesweeper
<video src="https://github.com/user-attachments/assets/425e2f81-81b0-4255-96dc-edf32fc4a60c"></video>

Max Richter - CGA Pong
<video src="https://github.com/user-attachments/assets/c36369b6-7603-46fc-a75e-e7a561addc48"></video>

### 2026
In 2026, hhuTOS was renamed to HeineOS.
It now works on modern 64-bit x86_64-computers with a UEFI BIOS and uses a framebuffer (provided by the bootloader) right from the start, instead of CGA text mode.

*Hall of Fame entries will follow soon.*
