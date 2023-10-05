#[derive(Debug, PartialEq)]
pub enum OpCode {
    CLR,              //clear screen
    JMP(u16),         //1NNN jmp to NNN
    RET,              //return from subroutine
    CALL(u16),        // call subroutine at nnn.
    SkipEqualNN(u8, u8), //3XNN skip next instruction if VX == NN
    SkipNotEqualNN(u8, u8), //4XNN skip next instruction if VX != NN
    SkipEqualXY(u8, u8),    //5XY0 skip next instruction if VX == VY
    SET(u8, u8),      //6XNN set register VX , X is addr in v_registers of 0-F
    ADD(u8, u8),      //7XNN add value to register VX
    LDXY(u8, u8),     //8XY0 set VX to value of VY
    BOR(u8, u8),      //8XY1 set VX to VX | VY
    BAND(u8, u8),     //8XY2 set VX to VX & VY
    BXOR(u8, u8),     //8XY3 set VX to VX ^ VY
    AddXY(u8, u8),    //8XY4 set VX to VX + VY, set VF to 1 if carry
    SubXY(u8, u8),    //8XY5 set VX to VX - VY, set VF to 0 if borrow
    SHR(u8, u8),          //8XY6 set VX to VX >> 1, set VF to least significant bit of VX before shift
    SUBN(u8, u8),     //8XY7 set VX to VY - VX, set VF to 0 if borrow
    SHL(u8, u8),          //8XYE set VX to VX << VY, set VF to most significant bit of VX before shift
    SkipNotEqualXY(u8, u8), //9XY0 skip next instruction if VX != VY
    SetAddrReg(u16),  //ANNN set index register I
    JumpPlusV0(u16),      //BNNN jump to NNN + V0
    RAND(u8, u8),     //CXNN set VX to random byte & NN
    DXYN(u8, u8, u8), //display/draw sprite at VX, VY with height N
    AddVxToI(u8),         //FX1E add VX to I
    SaveBCD(u8),      //FX33 store BCD representation of VX in memory locations I, I+1, I+2
    StoreV0ToVx(u8),      //FX55 save registers V0-VX to memory starting at I
    ReadV0ToVx(u8),      //FX65 load registers V0-VX from memory starting at I
    UNFINISHED,
}
