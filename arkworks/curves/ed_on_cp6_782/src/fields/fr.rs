use ark_ff::{
    biginteger::BigInteger384 as BigInteger,
    fields::{FftParameters, Fp384, Fp384Parameters, FpParameters},
};

pub type Fr = Fp384<FrParameters>;

pub struct FrParameters;

impl Fp384Parameters for FrParameters {}
impl FftParameters for FrParameters {
    type BigInt = BigInteger;

    const TWO_ADICITY: u32 = 2u32;

    #[rustfmt::skip]
    const TWO_ADIC_ROOT_OF_UNITY: BigInteger = BigInteger([
        12119792640622387781u64,
        8318439284650634613u64,
        6931324077796168275u64,
        12851391603681523141u64,
        6881015057611215092u64,
        1893962574900431u64,
    ]);
}
impl FpParameters for FrParameters {
    /// MODULUS = 32333053251621136751331591711861691692049189094364332567435817881934511297123972799646723302813083835942624121493
    #[rustfmt::skip]
    const MODULUS: BigInteger = BigInteger([
        4684667634276979349u64,
        3748803659444032385u64,
        16273581227874629698u64,
        7152942431629910641u64,
        6397188139321141543u64,
        15137289088311837u64,
    ]);

    const MODULUS_BITS: u32 = 374;

    const CAPACITY: u32 = Self::MODULUS_BITS - 1;

    const REPR_SHAVE_BITS: u32 = 10;

    #[rustfmt::skip]
    const R: BigInteger = BigInteger([
        12565484300600153878u64,
        8749673077137355528u64,
        9027943686469014788u64,
        13026065139386752555u64,
        11197589485989933721u64,
        9525964145733727u64,
    ]);

    #[rustfmt::skip]
    const R2: BigInteger = BigInteger([
        17257035094703902127u64,
        16096159112880350050u64,
        3498553494623421763u64,
        333405339929360058u64,
        1125865524035793947u64,
        1586246138566285u64,
    ]);

    const INV: u64 = 16242011933465909059u64;

    // 2
    #[rustfmt::skip]
    const GENERATOR: BigInteger = BigInteger([
        1999556893213776791u64,
        13750542494830678672u64,
        1782306145063399878u64,
        452443773434042853u64,
        15997990832658725900u64,
        3914639203155617u64,
    ]);

    #[rustfmt::skip]
    const MODULUS_MINUS_ONE_DIV_TWO: BigInteger = BigInteger([
        11565705853993265482u64,
        1874401829722016192u64,
        17360162650792090657u64,
        12799843252669731128u64,
        12421966106515346579u64,
        7568644544155918u64,
    ]);

    /// 8083263312905284187832897927965422923012297273591083141858954470483627824280993199911680825703270958985656030373
    const T: BigInteger = BigInteger([
        5782852926996632741,
        10160572951715783904,
        8680081325396045328,
        15623293663189641372,
        6210983053257673289,
        3784322272077959,
    ]);

    /// 4041631656452642093916448963982711461506148636795541570929477235241813912140496599955840412851635479492828015186
    const T_MINUS_ONE_DIV_TWO: BigInteger = BigInteger([
        2891426463498316370,
        5080286475857891952,
        4340040662698022664,
        17035018868449596494,
        12328863563483612452,
        1892161136038979,
    ]);
}
